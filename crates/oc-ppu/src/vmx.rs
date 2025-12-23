//! VMX/AltiVec SIMD support
//!
//! The Cell BE PPU includes VMX (Vector Multimedia Extensions),
//! also known as AltiVec, for SIMD operations.

/// VMX vector register (128-bit)
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, align(16))]
pub struct VmxRegister {
    pub data: [u8; 16],
}

impl VmxRegister {
    /// Create a new zero-initialized VMX register
    pub const fn new() -> Self {
        Self { data: [0; 16] }
    }

    /// Get as 4 x u32 (big-endian word order)
    pub fn as_u32x4(&self) -> [u32; 4] {
        [
            u32::from_be_bytes([self.data[0], self.data[1], self.data[2], self.data[3]]),
            u32::from_be_bytes([self.data[4], self.data[5], self.data[6], self.data[7]]),
            u32::from_be_bytes([self.data[8], self.data[9], self.data[10], self.data[11]]),
            u32::from_be_bytes([self.data[12], self.data[13], self.data[14], self.data[15]]),
        ]
    }

    /// Set from 4 x u32 (big-endian word order)
    pub fn set_u32x4(&mut self, values: [u32; 4]) {
        let b0 = values[0].to_be_bytes();
        let b1 = values[1].to_be_bytes();
        let b2 = values[2].to_be_bytes();
        let b3 = values[3].to_be_bytes();
        self.data = [
            b0[0], b0[1], b0[2], b0[3],
            b1[0], b1[1], b1[2], b1[3],
            b2[0], b2[1], b2[2], b2[3],
            b3[0], b3[1], b3[2], b3[3],
        ];
    }

    /// Get as 8 x u16 (big-endian halfword order)
    pub fn as_u16x8(&self) -> [u16; 8] {
        [
            u16::from_be_bytes([self.data[0], self.data[1]]),
            u16::from_be_bytes([self.data[2], self.data[3]]),
            u16::from_be_bytes([self.data[4], self.data[5]]),
            u16::from_be_bytes([self.data[6], self.data[7]]),
            u16::from_be_bytes([self.data[8], self.data[9]]),
            u16::from_be_bytes([self.data[10], self.data[11]]),
            u16::from_be_bytes([self.data[12], self.data[13]]),
            u16::from_be_bytes([self.data[14], self.data[15]]),
        ]
    }

    /// Get as 16 x u8
    pub fn as_u8x16(&self) -> [u8; 16] {
        self.data
    }

    /// Get as 4 x f32 (big-endian word order)
    pub fn as_f32x4(&self) -> [f32; 4] {
        let words = self.as_u32x4();
        [
            f32::from_bits(words[0]),
            f32::from_bits(words[1]),
            f32::from_bits(words[2]),
            f32::from_bits(words[3]),
        ]
    }

    /// Set from 4 x f32 (big-endian word order)
    pub fn set_f32x4(&mut self, values: [f32; 4]) {
        self.set_u32x4([
            values[0].to_bits(),
            values[1].to_bits(),
            values[2].to_bits(),
            values[3].to_bits(),
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vmx_register_u32x4() {
        let mut reg = VmxRegister::new();
        reg.set_u32x4([0x12345678, 0x9ABCDEF0, 0x11223344, 0x55667788]);
        
        let values = reg.as_u32x4();
        assert_eq!(values[0], 0x12345678);
        assert_eq!(values[1], 0x9ABCDEF0);
        assert_eq!(values[2], 0x11223344);
        assert_eq!(values[3], 0x55667788);
    }

    #[test]
    fn test_vmx_register_f32x4() {
        let mut reg = VmxRegister::new();
        reg.set_f32x4([1.0, 2.0, 3.0, 4.0]);
        
        let values = reg.as_f32x4();
        assert_eq!(values[0], 1.0);
        assert_eq!(values[1], 2.0);
        assert_eq!(values[2], 3.0);
        assert_eq!(values[3], 4.0);
    }
}
