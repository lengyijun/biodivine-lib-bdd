use crate::BddPointer;
use fxhash::FxHasher;
use std::hash::Hash;
use std::ops::{BitXor, Shl, BitOr, Shr};

/// The purpose of the dynamic op cache is to maintain a set of tasks
/// that need to be completed. It is essentially a hash set, but it
/// is optimized to trade between speed and uniqueness. I.e. a small number
/// of entries is allowed to collide and appear repeatedly as long as
/// the actual entry is fast.
pub(crate) struct DynamicOpCache {
    items: Vec<(u32, u32)>,
    /// Pointer into the items vector, assuming usize::MAX as undefined.
    /// When the number of collisions is at least 1/4,
    /// the whole thing is re-hashed.
    hashes: Vec<usize>,
    pub(crate) collisions_since_rehash: usize,
    /// Note that this can't be usize::MAX... we generally assume indices
    /// will not exceed u32 range.
    index_after_last_sorted_entry: usize,
    hasher: FxHasher,
}

const SEED64: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl DynamicOpCache {

    pub fn new(capacity: usize) -> DynamicOpCache {
        DynamicOpCache {
            items: Vec::with_capacity(capacity),
            hashes: vec![usize::MAX; capacity],
            collisions_since_rehash: 0,
            index_after_last_sorted_entry: 0,
            hasher: FxHasher::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[inline]
    pub(crate) fn contains(&self, l: BddPointer, r: BddPointer) -> bool {
        let hash = (hash(l.0, r.0) as usize) % self.hashes.len();
        let possible_index = self.hashes[hash];
        possible_index != usize::MAX && self.items[possible_index] == (l.0, r.0)
    }

    #[inline]
    pub(crate) fn contains2(&self, x: (BddPointer, BddPointer), y: (BddPointer, BddPointer)) -> (bool, bool) {
        let hash1 = (hash(x.0.0, x.1.0) as usize) % self.hashes.len();
        let hash2 = (hash(y.0.0, y.1.0) as usize) % self.hashes.len();
        let possible_index1 = self.hashes[hash1];
        let possible_index2 = self.hashes[hash2];
        (
            possible_index1 != usize::MAX && self.items[possible_index1] == (x.0.0, x.1.0),
            possible_index2 != usize::MAX && self.items[possible_index2] == (y.0.0, y.1.0)
        )
    }

    /// Returns true if new item is inserted, false if it already appears in the set.
    ///
    /// Note that this method can return a false negative result, i.e.
    /// re-insert entries that are already in the set. However, this should
    /// be relatively rare.
    #[inline]
    pub(crate) fn insert(&mut self, l: BddPointer, r: BddPointer) -> bool {
        let value = (l.0, r.0);
        let hash = (hash(value.0, value.1) as usize) % self.hashes.len();
        let possible_collision_at = self.hashes[hash];
        if possible_collision_at != usize::MAX {
            if self.items[possible_collision_at] == value {
                // No collision, just duplicate entry
                return false;
            }
            // Otherwise this is a collision.
            self.collisions_since_rehash += 1;
        }

        self.items.push(value);
        self.hashes[hash] = self.items.len() - 1;

        if self.collisions_since_rehash > self.items.len() >> 2 {
            self.rehash();
        }

        true
    }

    pub fn rehash(&mut self) {
        // First, sort the items that were inserted and merge them with existing items:
        self.items[self.index_after_last_sorted_entry..].sort();
        self.items = merge(
            &self.items[..self.index_after_last_sorted_entry],
            &self.items[self.index_after_last_sorted_entry..]
        );
        // Also update the index of the last sorted element.
        self.index_after_last_sorted_entry = self.items.len();

        // Now the whole thing is sorted and we can do the rehash into a new vector.
        self.collisions_since_rehash = 0;
        let mut hashes = vec![usize::MAX; self.hashes.len() * 2];
        for (i, (l, r)) in self.items.iter().enumerate() {
            let hash = (hash(*l, *r) as usize) % hashes.len();
            if hashes[hash] != usize::MAX {
                self.collisions_since_rehash += 1;
            }
            hashes[hash] = i;
        }
        self.hashes = hashes;
    }

}

#[inline]
fn hash(l: u32, r: u32) -> u64 {
    let packed: u64 = u64::from(l).shl(32) + u64::from(r);
    packed.wrapping_mul(SEED64)
}

/// Merge two sorted slices into one sorted vector.
///
/// Sadly, we can't really do this in place, but at least we try
/// to reserve so much capacity that the vector shouldn't need
/// reallocation until next rehash.
fn merge(left: &[(u32, u32)], right: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut result = Vec::with_capacity(2 * (left.len() + right.len()));
    let mut l = 0;
    let mut r = 0;
    while l < left.len() && r < right.len() {
        if left[l] < right[r] {
            result.push(left[l]);
            l += 1;
        } else {
            result.push(right[r]);
            r += 1;
        }
    }
    while l < left.len() {
        result.push(left[l]);
        l += 1;
    }
    while r < right.len() {
        result.push(right[r]);
        r += 1;
    }
    result
}

/*
fn pack(l: BddPointer, r: BddPointer) -> u64 {
    u64::from(l.0).shl(32).bitor(u64::from(r.0))
}

fn unpack(packed: u64) -> (BddPointer, BddPointer) {
    // Uses unsafe conversion because that is what is actually needed here.
    (BddPointer(packed.shr(32) as u32), BddPointer(packed as u32))
}*/