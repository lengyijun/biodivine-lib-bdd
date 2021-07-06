use crate::{BddPointer, Bdd};
use std::ops::{Shl, Shr, Rem, BitXor};
use std::num::{NonZeroU64, NonZeroU32};

pub(crate) struct Cache2 {
    pub collisions: usize,
    capacity: NonZeroU32,
    items: Vec<u32>,
}

impl Cache2 {

    pub fn new(capacity: usize) -> Cache2 {
        if capacity == 0 {
            panic!("FAIL");
        }
        Cache2 {
            collisions: 0,
            capacity: NonZeroU32::new(capacity as u32).unwrap(),
            // Setting mem to zero is ~0.5% faster
            items: vec![u32::MAX; capacity + 1],
        }
    }

    pub fn clear(&mut self) {
        self.collisions = 0;
        for i in self.items.iter_mut() {
            *i = u32::MAX;
        }
    }

    #[inline]
    pub fn contains(&self, l: BddPointer, r: BddPointer) -> bool {
        let packed = pack(l, r);
        let index = hash(packed).rem(self.capacity) as usize;
        unsafe {
            self.items.get_unchecked(index) == &packed
        }
    }

    #[inline]
    pub fn insert(&mut self, l: BddPointer, r: BddPointer) {
        let packed = pack(l, r);
        let index = (hash(packed) % self.capacity) as usize;
        unsafe { *self.items.get_unchecked_mut(index) = packed };
    }

}

const SEED64: u64 = 0x51_7c_c1_b7_27_22_0a_95;
const SEED32: u32 = 0x9e_37_79_b9;

#[inline]
fn hash(value: u32) -> u32 {
    value.wrapping_mul(SEED32)
}

#[inline]
fn pack(l: BddPointer, r: BddPointer) -> u32 {
    u32::from(l.0).shl(16) + u32::from(r.0)
}

/*
#[inline]
fn unpack(value: u64) -> (BddPointer, BddPointer) {
    (BddPointer(value.shr(32) as u32), BddPointer(value as u32))
}

#[cfg(test)]
mod tests {
    use crate::BddPointer;
    use crate::_impl_bdd::cache2::{unpack, pack};

    #[test]
    fn pack_unpack() {
        let (x,y) = (BddPointer(123456789), BddPointer(987654321));
        assert_eq!((x,y), unpack(pack(x,y)));
    }

}*/