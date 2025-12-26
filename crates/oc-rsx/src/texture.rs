//! RSX texture handling

use bitflags::bitflags;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

bitflags! {
    /// Texture format flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TextureFormat: u8 {
        const ARGB8 = 0x85;
        const DXT1 = 0x86;
        const DXT3 = 0x87;
        const DXT5 = 0x88;
        const A8R8G8B8 = 0x8A;
        const R5G6B5 = 0x8B;
    }
}

bitflags! {
    /// Texture filter modes
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TextureFilter: u8 {
        const NEAREST = 1;
        const LINEAR = 2;
    }
}

bitflags! {
    /// Texture wrap modes
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TextureWrap: u8 {
        const REPEAT = 1;
        const MIRRORED_REPEAT = 2;
        const CLAMP_TO_EDGE = 3;
        const CLAMP_TO_BORDER = 4;
    }
}

/// Texture descriptor
#[derive(Debug, Clone)]
pub struct Texture {
    /// GPU memory offset
    pub offset: u32,
    /// Texture format
    pub format: u8,
    /// Texture width
    pub width: u16,
    /// Texture height
    pub height: u16,
    /// Texture depth (for 3D textures)
    pub depth: u16,
    /// Number of mipmap levels
    pub mipmap_levels: u8,
    /// Texture pitch (stride)
    pub pitch: u16,
    /// Minification filter
    pub min_filter: TextureFilter,
    /// Magnification filter
    pub mag_filter: TextureFilter,
    /// Wrap mode U
    pub wrap_s: TextureWrap,
    /// Wrap mode V
    pub wrap_t: TextureWrap,
    /// Wrap mode W
    pub wrap_r: TextureWrap,
    /// Whether texture is cubemap
    pub is_cubemap: bool,
    /// Anisotropic filtering level (1.0 = disabled, 2.0, 4.0, 8.0, 16.0)
    pub anisotropy: f32,
    /// LOD bias
    pub lod_bias: f32,
    /// Minimum LOD level
    pub min_lod: f32,
    /// Maximum LOD level
    pub max_lod: f32,
}

impl Texture {
    /// Create a new texture
    pub fn new() -> Self {
        Self {
            offset: 0,
            format: 0,
            width: 0,
            height: 0,
            depth: 1,
            mipmap_levels: 1,
            pitch: 0,
            min_filter: TextureFilter::LINEAR,
            mag_filter: TextureFilter::LINEAR,
            wrap_s: TextureWrap::REPEAT,
            wrap_t: TextureWrap::REPEAT,
            wrap_r: TextureWrap::REPEAT,
            is_cubemap: false,
            anisotropy: 1.0,
            lod_bias: 0.0,
            min_lod: -1000.0,
            max_lod: 1000.0,
        }
    }

    /// Get size in bytes for this texture
    pub fn byte_size(&self) -> u32 {
        let bytes_per_pixel = match self.format {
            0x85 | 0x8A => 4, // ARGB8, A8R8G8B8
            0x8B => 2,         // R5G6B5
            0x86 => 1,         // DXT1 (0.5 bytes per pixel in 4x4 blocks = 8 bytes per block)
            0x87 | 0x88 => 1,  // DXT3/DXT5 (1 byte per pixel in 4x4 blocks = 16 bytes per block)
            _ => 4,
        };

        let mut size = 0u32;
        let mut w = self.width as u32;
        let mut h = self.height as u32;

        for _ in 0..self.mipmap_levels {
            if self.format == 0x86 || self.format == 0x87 || self.format == 0x88 {
                // Block-compressed formats: size in 4x4 blocks
                let blocks_w = w.div_ceil(4);
                let blocks_h = h.div_ceil(4);
                let block_size = if self.format == 0x86 { 8 } else { 16 };
                size += blocks_w * blocks_h * block_size;
            } else {
                size += w * h * bytes_per_pixel;
            }
            w = (w / 2).max(1);
            h = (h / 2).max(1);
        }

        if self.is_cubemap {
            size *= 6; // 6 faces for cubemap
        }

        size
    }
}

impl Default for Texture {
    fn default() -> Self {
        Self::new()
    }
}

/// Texture cache for storing uploaded texture data
pub struct TextureCache {
    /// Cached textures
    textures: Vec<CachedTexture>,
    /// Maximum cache size in bytes
    max_size: usize,
    /// Current cache size
    current_size: usize,
}

/// A cached texture entry
#[derive(Clone)]
struct CachedTexture {
    /// GPU memory offset
    offset: u32,
    /// Texture descriptor
    descriptor: Texture,
    /// Cached texture data
    data: Vec<u8>,
    /// Last access timestamp
    last_used: u64,
}

impl TextureCache {
    /// Create a new texture cache
    pub fn new(max_size: usize) -> Self {
        Self {
            textures: Vec::new(),
            max_size,
            current_size: 0,
        }
    }

    /// Get cached texture
    pub fn get(&mut self, offset: u32, timestamp: u64) -> Option<(&Texture, &[u8])> {
        if let Some(cached) = self.textures.iter_mut().find(|t| t.offset == offset) {
            cached.last_used = timestamp;
            Some((&cached.descriptor, cached.data.as_slice()))
        } else {
            None
        }
    }

    /// Insert texture into cache
    pub fn insert(&mut self, offset: u32, descriptor: Texture, data: Vec<u8>, timestamp: u64) {
        // Remove existing entry if present
        if let Some(pos) = self.textures.iter().position(|t| t.offset == offset) {
            let old = self.textures.remove(pos);
            self.current_size -= old.data.len();
        }

        let data_len = data.len();
        self.textures.push(CachedTexture {
            offset,
            descriptor,
            data,
            last_used: timestamp,
        });
        self.current_size += data_len;

        // Evict least recently used entries if cache is full
        while self.current_size > self.max_size && !self.textures.is_empty() {
            if let Some(lru_pos) = self.find_lru() {
                let old = self.textures.remove(lru_pos);
                self.current_size -= old.data.len();
            }
        }
    }

    /// Find least recently used texture
    fn find_lru(&self) -> Option<usize> {
        self.textures
            .iter()
            .enumerate()
            .min_by_key(|(_, t)| t.last_used)
            .map(|(i, _)| i)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.textures.clear();
        self.current_size = 0;
    }

    /// Invalidate entries at or after offset
    pub fn invalidate(&mut self, offset: u32) {
        self.textures.retain(|t| {
            if t.offset >= offset {
                self.current_size -= t.data.len();
                false
            } else {
                true
            }
        });
    }

    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.textures.len(), self.current_size, self.max_size)
    }
}

/// Texture sampler configuration for accurate sampling
#[derive(Debug, Clone)]
pub struct TextureSampler {
    /// Minification filter
    pub min_filter: TextureFilter,
    /// Magnification filter
    pub mag_filter: TextureFilter,
    /// Mipmap filter
    pub mipmap_filter: TextureFilter,
    /// Wrap mode U
    pub wrap_s: TextureWrap,
    /// Wrap mode V
    pub wrap_t: TextureWrap,
    /// Wrap mode W
    pub wrap_r: TextureWrap,
    /// Anisotropic filtering level (1.0-16.0)
    pub max_anisotropy: f32,
    /// LOD bias
    pub lod_bias: f32,
    /// Minimum LOD
    pub min_lod: f32,
    /// Maximum LOD
    pub max_lod: f32,
    /// Border color (RGBA)
    pub border_color: [f32; 4],
    /// Compare mode for depth textures
    pub compare_enable: bool,
    /// Compare function
    pub compare_func: u32,
}

impl TextureSampler {
    /// Create a new texture sampler with default settings
    pub fn new() -> Self {
        Self {
            min_filter: TextureFilter::LINEAR,
            mag_filter: TextureFilter::LINEAR,
            mipmap_filter: TextureFilter::LINEAR,
            wrap_s: TextureWrap::REPEAT,
            wrap_t: TextureWrap::REPEAT,
            wrap_r: TextureWrap::REPEAT,
            max_anisotropy: 1.0,
            lod_bias: 0.0,
            min_lod: -1000.0,
            max_lod: 1000.0,
            border_color: [0.0, 0.0, 0.0, 0.0],
            compare_enable: false,
            compare_func: 0, // NEVER
        }
    }

    /// Create a sampler with anisotropic filtering
    pub fn with_anisotropy(mut self, level: f32) -> Self {
        self.max_anisotropy = level.clamp(1.0, 16.0);
        self
    }

    /// Create a sampler with LOD bias
    pub fn with_lod_bias(mut self, bias: f32) -> Self {
        self.lod_bias = bias;
        self
    }

    /// Create a sampler with LOD range
    pub fn with_lod_range(mut self, min: f32, max: f32) -> Self {
        self.min_lod = min;
        self.max_lod = max;
        self
    }

    /// Create a sampler for depth comparison
    pub fn with_compare(mut self, func: u32) -> Self {
        self.compare_enable = true;
        self.compare_func = func;
        self
    }

    /// Apply sampler configuration to a texture
    pub fn apply_to_texture(&self, texture: &mut Texture) {
        texture.min_filter = self.min_filter;
        texture.mag_filter = self.mag_filter;
        texture.wrap_s = self.wrap_s;
        texture.wrap_t = self.wrap_t;
        texture.wrap_r = self.wrap_r;
        texture.anisotropy = self.max_anisotropy;
        texture.lod_bias = self.lod_bias;
        texture.min_lod = self.min_lod;
        texture.max_lod = self.max_lod;
    }
}

impl Default for TextureSampler {
    fn default() -> Self {
        Self::new()
    }
}

/// Asynchronous texture loading system
pub struct AsyncTextureLoader {
    /// Sender for texture load requests
    request_sender: Sender<TextureLoadRequest>,
    /// Receiver for loaded textures
    result_receiver: Receiver<TextureLoadResult>,
    /// Number of worker threads
    worker_count: usize,
}

/// Texture load request
#[derive(Clone)]
struct TextureLoadRequest {
    /// Texture ID
    id: u64,
    /// GPU memory offset
    offset: u32,
    /// Texture descriptor
    descriptor: Texture,
}

/// Texture load result
#[derive(Clone)]
struct TextureLoadResult {
    /// Texture ID
    id: u64,
    /// GPU memory offset
    offset: u32,
    /// Texture descriptor
    descriptor: Texture,
    /// Loaded texture data
    data: Vec<u8>,
    /// Whether loading succeeded
    success: bool,
}

impl AsyncTextureLoader {
    /// Create a new async texture loader
    pub fn new(worker_count: usize) -> Self {
        let (request_sender, request_receiver) = channel::<TextureLoadRequest>();
        let (result_sender, result_receiver) = channel::<TextureLoadResult>();

        // Spawn worker threads using shared Arc for receiver
        use std::sync::{Arc, Mutex};
        let shared_rx = Arc::new(Mutex::new(request_receiver));

        for _ in 0..worker_count {
            let rx = Arc::clone(&shared_rx);
            let tx = result_sender.clone();

            thread::spawn(move || {
                loop {
                    let request = {
                        let locked = rx.lock().unwrap();
                        locked.recv()
                    };

                    match request {
                        Ok(req) => {
                            // Simulate texture loading
                            let size = req.descriptor.byte_size() as usize;
                            let data = vec![0u8; size]; // In real implementation, would read from memory
                            
                            let result = TextureLoadResult {
                                id: req.id,
                                offset: req.offset,
                                descriptor: req.descriptor,
                                data,
                                success: true,
                            };

                            if tx.send(result).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        Self {
            request_sender,
            result_receiver,
            worker_count,
        }
    }

    /// Request to load a texture asynchronously
    pub fn load_async(&self, id: u64, offset: u32, descriptor: Texture) -> Result<(), String> {
        let request = TextureLoadRequest {
            id,
            offset,
            descriptor,
        };

        self.request_sender
            .send(request)
            .map_err(|e| format!("Failed to send load request: {}", e))
    }

    /// Check for completed texture loads
    pub fn poll_completed(&self) -> Vec<(u64, u32, Texture, Vec<u8>)> {
        let mut results = Vec::new();
        
        while let Ok(result) = self.result_receiver.try_recv() {
            if result.success {
                results.push((result.id, result.offset, result.descriptor, result.data));
            }
        }

        results
    }

    /// Get number of worker threads
    pub fn worker_count(&self) -> usize {
        self.worker_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_byte_size() {
        let mut tex = Texture::new();
        tex.width = 256;
        tex.height = 256;
        tex.format = 0x8A; // A8R8G8B8
        tex.mipmap_levels = 1;
        assert_eq!(tex.byte_size(), 256 * 256 * 4);
    }

    #[test]
    fn test_texture_cache() {
        let mut cache = TextureCache::new(1000);

        let tex = Texture::new();
        let data = vec![1, 2, 3, 4];
        cache.insert(0x1000, tex, data.clone(), 1);

        let (_, cached_data) = cache.get(0x1000, 2).unwrap();
        assert_eq!(cached_data, data.as_slice());
    }

    #[test]
    fn test_texture_cache_eviction() {
        let mut cache = TextureCache::new(20);

        let tex1 = Texture::new();
        cache.insert(0x1000, tex1, vec![0; 15], 1);

        let tex2 = Texture::new();
        cache.insert(0x2000, tex2, vec![0; 15], 2);

        // Should evict first texture (LRU)
        assert!(cache.get(0x1000, 3).is_none());
        assert!(cache.get(0x2000, 3).is_some());
    }

    #[test]
    fn test_texture_cache_invalidate() {
        let mut cache = TextureCache::new(1000);

        cache.insert(0x1000, Texture::new(), vec![1, 2, 3], 1);
        cache.insert(0x2000, Texture::new(), vec![4, 5, 6], 2);

        cache.invalidate(0x1500);

        assert!(cache.get(0x1000, 3).is_some());
        assert!(cache.get(0x2000, 3).is_none());
    }

    #[test]
    fn test_texture_anisotropy() {
        let mut tex = Texture::new();
        tex.anisotropy = 16.0;
        assert_eq!(tex.anisotropy, 16.0);
    }

    #[test]
    fn test_texture_lod() {
        let mut tex = Texture::new();
        tex.lod_bias = 0.5;
        tex.min_lod = 0.0;
        tex.max_lod = 10.0;
        assert_eq!(tex.lod_bias, 0.5);
        assert_eq!(tex.min_lod, 0.0);
        assert_eq!(tex.max_lod, 10.0);
    }
}
