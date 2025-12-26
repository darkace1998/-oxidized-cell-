//! RSX vertex processing

use bitflags::bitflags;

bitflags! {
    /// Vertex attribute type flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct VertexAttributeType: u8 {
        const FLOAT = 1;
        const SHORT = 2;
        const BYTE = 3;
        const HALF_FLOAT = 4;
        const COMPRESSED = 5;
    }
}

/// Vertex attribute descriptor
#[derive(Debug, Clone, Copy)]
pub struct VertexAttribute {
    /// Attribute index (0-15)
    pub index: u8,
    /// Number of components (1-4)
    pub size: u8,
    /// Data type
    pub type_: VertexAttributeType,
    /// Stride between vertices
    pub stride: u16,
    /// Offset into vertex data
    pub offset: u32,
    /// Whether attribute is normalized
    pub normalized: bool,
}

impl VertexAttribute {
    /// Create a new vertex attribute
    pub fn new(index: u8) -> Self {
        Self {
            index,
            size: 4,
            type_: VertexAttributeType::FLOAT,
            stride: 0,
            offset: 0,
            normalized: false,
        }
    }

    /// Get size in bytes for this attribute
    pub fn byte_size(&self) -> u32 {
        let type_size = match self.type_ {
            VertexAttributeType::FLOAT => 4,
            VertexAttributeType::SHORT => 2,
            VertexAttributeType::BYTE => 1,
            VertexAttributeType::HALF_FLOAT => 2,
            VertexAttributeType::COMPRESSED => 4,
            _ => {
                tracing::warn!("Unknown vertex attribute type, defaulting to 4 bytes");
                4
            }
        };
        (self.size as u32) * type_size
    }
}

/// Vertex buffer descriptor
#[derive(Debug, Clone)]
pub struct VertexBuffer {
    /// GPU memory address
    pub address: u32,
    /// Buffer size in bytes
    pub size: u32,
    /// Vertex stride
    pub stride: u16,
}

impl VertexBuffer {
    /// Create a new vertex buffer
    pub fn new(address: u32, size: u32, stride: u16) -> Self {
        Self { address, size, stride }
    }
}

/// Vertex cache for storing processed vertex data
pub struct VertexCache {
    /// Cached vertex buffers
    buffers: Vec<(u32, Vec<u8>)>, // (address, data)
    /// Maximum cache size in bytes
    max_size: usize,
    /// Current cache size
    current_size: usize,
}

impl VertexCache {
    /// Create a new vertex cache
    pub fn new(max_size: usize) -> Self {
        Self {
            buffers: Vec::new(),
            max_size,
            current_size: 0,
        }
    }

    /// Get cached vertex data
    pub fn get(&self, address: u32) -> Option<&[u8]> {
        self.buffers
            .iter()
            .find(|(addr, _)| *addr == address)
            .map(|(_, data)| data.as_slice())
    }

    /// Insert vertex data into cache
    pub fn insert(&mut self, address: u32, data: Vec<u8>) {
        // Remove existing entry if present
        if let Some(pos) = self.buffers.iter().position(|(addr, _)| *addr == address) {
            let (_, old_data) = self.buffers.remove(pos);
            self.current_size -= old_data.len();
        }

        self.current_size += data.len();
        self.buffers.push((address, data));

        // Evict oldest entries if cache is full
        while self.current_size > self.max_size && !self.buffers.is_empty() {
            let (_, old_data) = self.buffers.remove(0);
            self.current_size -= old_data.len();
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.buffers.clear();
        self.current_size = 0;
    }

    /// Invalidate entries at or after address
    pub fn invalidate(&mut self, address: u32) {
        self.buffers.retain(|(addr, data)| {
            if *addr >= address {
                self.current_size -= data.len();
                false
            } else {
                true
            }
        });
    }
}

/// Post-transform vertex cache for optimization
pub struct PostTransformVertexCache {
    /// Cache entries (index -> vertex data hash)
    entries: Vec<Option<u64>>,
    /// Cache size (typically 16-32 for hardware)
    size: usize,
    /// Next insertion position (FIFO)
    next_pos: usize,
    /// Cache hits counter
    hits: usize,
    /// Cache misses counter
    misses: usize,
}

impl PostTransformVertexCache {
    /// Create a new post-transform vertex cache
    pub fn new(size: usize) -> Self {
        Self {
            entries: vec![None; size],
            size,
            next_pos: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Check if vertex is in cache
    pub fn lookup(&mut self, vertex_hash: u64) -> bool {
        if self.entries.iter().any(|entry| entry == &Some(vertex_hash)) {
            self.hits += 1;
            true
        } else {
            self.misses += 1;
            false
        }
    }

    /// Insert vertex into cache
    pub fn insert(&mut self, vertex_hash: u64) {
        self.entries[self.next_pos] = Some(vertex_hash);
        self.next_pos = (self.next_pos + 1) % self.size;
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.entries.fill(None);
        self.next_pos = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize, f32) {
        let total = self.hits + self.misses;
        let hit_rate = if total > 0 {
            self.hits as f32 / total as f32
        } else {
            0.0
        };
        (self.hits, self.misses, hit_rate)
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }
}

/// Optimized vertex processor with batching and caching
pub struct VertexProcessor {
    /// Post-transform vertex cache
    pt_cache: PostTransformVertexCache,
    /// Vertex attribute descriptors
    attributes: Vec<VertexAttribute>,
    /// Batch size for processing
    batch_size: usize,
}

impl VertexProcessor {
    /// Create a new vertex processor
    pub fn new(cache_size: usize, batch_size: usize) -> Self {
        Self {
            pt_cache: PostTransformVertexCache::new(cache_size),
            attributes: Vec::new(),
            batch_size,
        }
    }

    /// Set vertex attributes
    pub fn set_attributes(&mut self, attributes: Vec<VertexAttribute>) {
        self.attributes = attributes;
    }

    /// Process vertex data with optimization
    pub fn process_vertices(&mut self, indices: &[u32], vertex_data: &[u8]) -> Vec<u8> {
        let mut processed = Vec::new();
        
        for &index in indices {
            let vertex_hash = self.compute_vertex_hash(index, vertex_data);
            
            if !self.pt_cache.lookup(vertex_hash) {
                // Cache miss - need to process vertex
                let vertex = self.extract_vertex(index, vertex_data);
                processed.extend_from_slice(&vertex);
                self.pt_cache.insert(vertex_hash);
            } else {
                // Cache hit - reuse transformed vertex
                // In a real implementation, we'd fetch the transformed vertex
                let vertex = self.extract_vertex(index, vertex_data);
                processed.extend_from_slice(&vertex);
            }
        }

        processed
    }

    /// Compute hash for a vertex
    fn compute_vertex_hash(&self, index: u32, _vertex_data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        index.hash(&mut hasher);
        hasher.finish()
    }

    /// Extract vertex data at index
    fn extract_vertex(&self, index: u32, vertex_data: &[u8]) -> Vec<u8> {
        let mut vertex = Vec::new();
        
        for attr in &self.attributes {
            let offset = (index * attr.stride as u32 + attr.offset) as usize;
            let size = attr.byte_size() as usize;
            
            if offset + size <= vertex_data.len() {
                vertex.extend_from_slice(&vertex_data[offset..offset + size]);
            }
        }

        vertex
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize, f32) {
        self.pt_cache.stats()
    }

    /// Clear caches
    pub fn clear(&mut self) {
        self.pt_cache.clear();
    }

    /// Get batch size
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_attribute_byte_size() {
        let mut attr = VertexAttribute::new(0);
        attr.type_ = VertexAttributeType::FLOAT;
        attr.size = 3;
        assert_eq!(attr.byte_size(), 12); // 3 floats * 4 bytes

        attr.type_ = VertexAttributeType::SHORT;
        attr.size = 2;
        assert_eq!(attr.byte_size(), 4); // 2 shorts * 2 bytes
    }

    #[test]
    fn test_vertex_cache() {
        let mut cache = VertexCache::new(100);

        let data1 = vec![1, 2, 3, 4];
        cache.insert(0x1000, data1.clone());
        
        assert_eq!(cache.get(0x1000), Some(data1.as_slice()));
        assert_eq!(cache.get(0x2000), None);

        cache.clear();
        assert_eq!(cache.get(0x1000), None);
    }

    #[test]
    fn test_vertex_cache_eviction() {
        let mut cache = VertexCache::new(8);

        cache.insert(0x1000, vec![1, 2, 3, 4, 5]);
        cache.insert(0x2000, vec![6, 7, 8, 9, 10]);
        
        // Should evict first entry since 5 + 5 > 8
        assert_eq!(cache.get(0x1000), None);
        assert!(cache.get(0x2000).is_some());
    }
}
