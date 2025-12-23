//! Shared types for FFI

/// 128-bit value for vector operations
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Default)]
pub struct V128 {
    pub data: [u8; 16],
}

impl V128 {
    pub const fn new() -> Self {
        Self { data: [0; 16] }
    }

    pub fn from_u32x4(values: [u32; 4]) -> Self {
        let mut data = [0u8; 16];
        for (i, v) in values.iter().enumerate() {
            let bytes = v.to_ne_bytes();
            data[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
        }
        Self { data }
    }

    pub fn to_u32x4(&self) -> [u32; 4] {
        [
            u32::from_ne_bytes([self.data[0], self.data[1], self.data[2], self.data[3]]),
            u32::from_ne_bytes([self.data[4], self.data[5], self.data[6], self.data[7]]),
            u32::from_ne_bytes([self.data[8], self.data[9], self.data[10], self.data[11]]),
            u32::from_ne_bytes([self.data[12], self.data[13], self.data[14], self.data[15]]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v128_conversion() {
        let values = [0x11111111u32, 0x22222222, 0x33333333, 0x44444444];
        let v = V128::from_u32x4(values);
        assert_eq!(v.to_u32x4(), values);
    }
}
