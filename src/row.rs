//! Implements access to a matrix's individual rows.

use std::mem;
use std::ops::{Deref, DerefMut};
use std::ops::Index;
use std::ops::Range;

use util::div_rem;
use super::{Block, BITS, TRUE, FALSE};

/// A slice of bit vector's blocks.
pub struct BitVecSlice {
    slice: [Block],
}

impl BitVecSlice {
    /// Creates a new slice from a slice of blocks.
    #[inline]
    pub fn new(slice: &[Block]) -> &Self {
        unsafe {
            mem::transmute(slice)
        }
    }

    /// Iterates over bits.
    #[inline]
    pub fn iter_bits(&self, len: usize) -> Iter {
        Iter { bit_slice: self, range: 0..len }
    }

    /// Returns `true` if a bit is enabled in the bit vector slice, or `false` otherwise.
    #[inline]
    pub fn get(&self, bit: usize) -> bool {
        let (block, i) = div_rem(bit, BITS);
        match self.slice.get(block) {
            None => false,
            Some(b) => (b & (1 << i)) != 0,
        }
    }
}

/// Returns `true` if a bit is enabled in the bit vector slice, or `false` otherwise.
impl Index<usize> for BitVecSlice {
    type Output = bool;

    #[inline]
    fn index(&self, bit: usize) -> &bool {
        let (block, i) = div_rem(bit, BITS);
        match self.slice.get(block) {
            None => &FALSE,
            Some(b) => if (b & (1 << i)) != 0 { &TRUE } else { &FALSE },
        }
    }
}

impl Deref for BitVecSlice {
    type Target = [Block];

    #[inline]
    fn deref(&self) -> &[Block] {
        &self.slice
    }
}

impl DerefMut for BitVecSlice {
    #[inline]
    fn deref_mut(&mut self) -> &mut [Block] {
        &mut self.slice
    }
}

/// An iterator for `BitVecSlice`.
#[derive(Clone)]
pub struct Iter<'a> {
    bit_slice: &'a BitVecSlice,
    range: Range<usize>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<bool> {
        self.range.next().map(|i| self.bit_slice[i])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}
