//! FNV-1a 64 for event logs and state hashes. Written here so the hash never changes under us.

const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// Incremental FNV-1a 64-bit hasher.
#[derive(Debug, Clone, Copy)]
pub struct Fnv64(u64);

impl Fnv64 {
    /// Hasher over no bytes yet.
    #[must_use]
    pub const fn new() -> Self {
        Self(OFFSET_BASIS)
    }

    /// Feeds raw bytes.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(PRIME);
        }
    }

    /// Feeds one byte.
    pub fn write_u8(&mut self, v: u8) {
        self.write_bytes(&[v]);
    }

    /// Feeds a `u16` as little-endian bytes.
    pub fn write_u16(&mut self, v: u16) {
        self.write_bytes(&v.to_le_bytes());
    }

    /// Feeds a `u32` as little-endian bytes.
    pub fn write_u32(&mut self, v: u32) {
        self.write_bytes(&v.to_le_bytes());
    }

    /// Feeds an `f32` by its exact bit pattern, so `0.0` and `-0.0` differ.
    pub fn write_f32(&mut self, v: f32) {
        self.write_u32(v.to_bits());
    }

    /// The hash so far.
    #[must_use]
    pub const fn finish(self) -> u64 {
        self.0
    }
}

impl Default for Fnv64 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Fnv64;

    #[test]
    fn empty_input_is_the_offset_basis() {
        assert_eq!(Fnv64::new().finish(), 0xcbf2_9ce4_8422_2325);
    }

    #[test]
    fn matches_the_published_vector_for_a() {
        let mut h = Fnv64::new();
        h.write_bytes(b"a");
        assert_eq!(h.finish(), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn floats_hash_by_bit_pattern() {
        let mut a = Fnv64::new();
        a.write_f32(1.5);
        let mut b = Fnv64::new();
        b.write_u32(1.5f32.to_bits());
        assert_eq!(a.finish(), b.finish());
        let mut zero = Fnv64::new();
        zero.write_f32(0.0);
        let mut neg = Fnv64::new();
        neg.write_f32(-0.0);
        assert_ne!(zero.finish(), neg.finish());
    }
}
