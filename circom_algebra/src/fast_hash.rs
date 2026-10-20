//! A fast, deterministic, non-cryptographic hasher (FxHash, as used by rustc).
//!
//! The compiler's internal tables are keyed by small integers (signal ids) and by
//! `String`s that we control, so the DoS resistance of the standard library's
//! SipHash-1-3 buys us nothing while costing a measurable share of compile time
//! (~13% of the simplification phase was spent hashing). These aliases keep the
//! `std` collection APIs but swap in FxHash.

use std::hash::{BuildHasherDefault, Hasher};

const SEED64: u64 = 0x51_7c_c1_b7_27_22_0a_95;

#[derive(Default, Clone, Copy)]
pub struct FxHasher {
    hash: u64,
}

impl FxHasher {
    #[inline]
    fn add_to_hash(&mut self, i: u64) {
        self.hash = (self.hash.rotate_left(5) ^ i).wrapping_mul(SEED64);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut rest = bytes;
        while rest.len() >= 8 {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&rest[..8]);
            self.add_to_hash(u64::from_ne_bytes(buf));
            rest = &rest[8..];
        }
        if rest.len() >= 4 {
            let mut buf = [0u8; 4];
            buf.copy_from_slice(&rest[..4]);
            self.add_to_hash(u32::from_ne_bytes(buf) as u64);
            rest = &rest[4..];
        }
        for b in rest {
            self.add_to_hash(*b as u64);
        }
    }
    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add_to_hash(i as u64);
    }
    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add_to_hash(i as u64);
    }
    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add_to_hash(i as u64);
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i);
    }
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add_to_hash(i as u64);
    }
    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

pub type FxBuildHasher = BuildHasherDefault<FxHasher>;
pub type HashMap<K, V> = std::collections::HashMap<K, V, FxBuildHasher>;
pub type HashSet<K> = std::collections::HashSet<K, FxBuildHasher>;

#[inline]
pub fn new_map<K, V>() -> HashMap<K, V> {
    HashMap::default()
}

#[inline]
pub fn map_with_capacity<K, V>(capacity: usize) -> HashMap<K, V> {
    HashMap::with_capacity_and_hasher(capacity, FxBuildHasher::default())
}

#[inline]
pub fn new_set<K>() -> HashSet<K> {
    HashSet::default()
}

#[inline]
pub fn set_with_capacity<K>(capacity: usize) -> HashSet<K> {
    HashSet::with_capacity_and_hasher(capacity, FxBuildHasher::default())
}
