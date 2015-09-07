//! Bit matrices and vectors.

#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

extern crate bit_vec;

use std::fmt;
use std::iter::Map;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ops::{Index, IndexMut};
use std::ops::Range;
use std::slice;

use bit_vec::BitVec;

static TRUE: bool = true;
static FALSE: bool = false;

const BITS: usize = 32;
/// The type for storing bits.
pub type Block = u32;

/// A simple fixed-size vector of bits.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedBitVec {
    data: Box<[Block]>,
    /// length in bits
    length: usize,
}

/// A fixed-size matrix of bits.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedBitMatrix {
    bit_vec: FixedBitVec,
    row_bits: usize,
}

/// A matrix of bits.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitMatrix {
    bit_vec: BitVec,
    row_bits: usize,
}

/// Immutable access to a range of matrix's rows.
pub struct BitSubMatrix<'a> {
    slice: &'a [Block],
    row_bits: usize,
}

/// Mutable access to a range of matrix's rows.
pub struct BitSubMatrixMut<'a> {
    slice: &'a mut [Block],
    row_bits: usize,
}

/// A slice of bit vector's blocks.
pub struct BitVecSlice {
    slice: [Block],
}

impl FixedBitVec {
    /// Create an empty FixedBitVec.
    pub fn new() -> Self {
        FixedBitVec::from_elem(0, false)
    }

    /// Create a new FixedBitVec with a specific number of bits.
    pub fn from_elem(bits: usize, elem: bool) -> Self {
        let blocks = round_up_to_next(bits, BITS) / BITS;
        let mut data = Vec::with_capacity(blocks);
        unsafe {
            data.set_len(blocks);
            let elem = if elem { !0 } else { 0 };
            for block in &mut data {
                *block = elem;
            }
        }
        FixedBitVec {
            data: data.into_boxed_slice(),
            length: bits,
        }
    }

    #[inline]
    /// Return the vector's length in bits.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns the bit's value in the FixedBitVec.
    ///
    /// Note: the returned value is unspecified if the last block's padding is accessed.
    #[inline]
    pub fn get(&self, bit: usize) -> Option<bool> {
        let (block, i) = div_rem(bit, BITS);
        match self.data.get(block) {
            None => None,
            Some(b) => Some((b & (1 << i)) != 0),
        }
    }

    /// Clear all bits.
    #[inline]
    pub fn clear(&mut self) {
        for elem in &mut self.data[..] {
            *elem = 0;
        }
    }

    /// Sets the value of a bit.
    ///
    /// # Panics
    ///
    /// Panics if the `bit` index is out of bounds.
    #[inline]
    pub fn set(&mut self, bit: usize, enabled: bool) {
        assert!(bit < self.length);
        let (block, i) = div_rem(bit, BITS);
        unsafe {
            let elt = self.data.get_unchecked_mut(block);
            if enabled {
                *elt |= 1 << i;
            } else {
                *elt &= !(1 << i);
            }
        }
    }

    /// Exposes the block storage of the FixedBitVec.
    #[inline]
    pub fn storage(&self) -> &[u32] {
        &self.data
    }

    /// Exposes the block storage of the FixedBitVec.
    #[inline]
    pub fn storage_mut(&mut self) -> &mut [u32] {
        &mut self.data
    }

    /// Creates a new FixedBitVec from the given BitVec.
    pub fn from_bit_vec(mut bit_vec: BitVec) -> FixedBitVec {
        unsafe {
            FixedBitVec {
                data: mem::replace(bit_vec.storage_mut(), Vec::new()).into_boxed_slice(),
                length: bit_vec.len(),
            }
        }
    }

    /// Returns an iterator over bits.
    #[inline]
    pub fn iter<'a>(&'a self) -> Iter<'a> {
        unsafe {
            Iter { bit_slice: mem::transmute(&*self.data), range: 0..self.length }
        }
    }
}

impl Clone for FixedBitVec {
    fn clone(&self) -> Self {
        FixedBitVec {
            data: self.data.to_vec().into_boxed_slice(),
            length: self.length,
        }
    }
}

// Matrix

impl BitMatrix {
    /// Create a new BitMatrix with specific numbers of bits in columns and rows.
    pub fn new(rows: usize, row_bits: usize) -> Self {
        BitMatrix {
            bit_vec: BitVec::from_elem(round_up_to_next(row_bits, BITS) * rows, false),
            row_bits: row_bits,
        }
    }

    /// Returns the number of rows.
    #[inline]
    fn num_rows(&self) -> usize {
        let row_blocks = round_up_to_next(self.row_bits, BITS) / BITS;
        self.bit_vec.storage().len() / row_blocks
    }

    /// Returns the matrix's size as `(rows, columns)`.
    pub fn size(&self) -> (usize, usize) {
        (self.num_rows(), self.row_bits)
    }

    /// Converts the matrix into a fixed-size matrix.
    pub fn into_fixed(self) -> FixedBitMatrix {
        FixedBitMatrix {
            bit_vec: FixedBitVec::from_bit_vec(self.bit_vec),
            row_bits: self.row_bits,
        }
    }

    /// Grows the matrix in-place, adding `num_rows` rows filled with `value`.
    pub fn grow(&mut self, num_rows: usize, value: bool) {
        self.bit_vec.grow(round_up_to_next(self.row_bits, BITS) * num_rows, value);
    }
}

impl FixedBitMatrix {
    /// Create a new FixedBitMatrix with specific numbers of bits in columns and rows.
    pub fn new(rows: usize, row_bits: usize) -> Self {
        FixedBitMatrix {
            bit_vec: FixedBitVec::from_elem(round_up_to_next(row_bits, BITS) * rows, false),
            row_bits: row_bits,
        }
    }

    /// Returns the number of rows.
    #[inline]
    fn num_rows(&self) -> usize {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        self.bit_vec.storage().len() / row_size
    }

    /// Returns the matrix's size as `(rows, columns)`.
    pub fn size(&self) -> (usize, usize) {
        (self.num_rows(), self.row_bits)
    }

    /// Sets the value of a bit.
    ///
    /// # Panics
    ///
    /// Panics if `(row, col)` is out of bounds.
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, enabled: bool) {
        let row_size_in_bits = round_up_to_next(self.row_bits, BITS);
        self.bit_vec.set(row * row_size_in_bits + col, enabled);
    }

    /// Returns a slice of the matrix's rows.
    #[inline]
    pub fn sub_matrix(&self, range: Range<usize>) -> BitSubMatrix {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        BitSubMatrix {
            slice: &self.bit_vec.storage()[range.start * row_size .. range.end * row_size],
            row_bits: self.row_bits,
        }
    }

    /// Given a row's index, returns a slice of all rows above that row, a reference to said row,
    /// and a slice of all rows below.
    ///
    /// Functionally equivalent to `(self.sub_matrix(0..row), &self[row],
    /// self.sub_matrix(row..self.num_rows()))`.
    #[inline]
    pub fn split_at(&self, row: usize)
                    -> (BitSubMatrix,
                        &BitVecSlice,
                        BitSubMatrixMut) {
        unsafe {
            (mem::transmute(self.sub_matrix(0 .. row)),
             mem::transmute(&self[row]),
             mem::transmute(self.sub_matrix(row + 1 .. self.num_rows())))
        }
    }

    /// Given a row's index, returns a slice of all rows above that row, a reference to said row,
    /// and a slice of all rows below.
    #[inline]
    pub fn split_at_mut(&mut self, row: usize)
                        -> (BitSubMatrixMut,
                            &mut BitVecSlice,
                            BitSubMatrixMut) {
        unsafe {
            (mem::transmute(self.sub_matrix(0 .. row)),
             mem::transmute(&mut self[row]),
             mem::transmute(self.sub_matrix(row + 1 .. self.num_rows())))
        }
    }

    /// Iterate over bits in the specified row.
    pub fn iter_row(&self, row: usize) -> Iter {
        Iter { bit_slice: &self[row], range: 0..self.row_bits }
    }

    /// Computes the transitive closure of the binary relation represented by the matrix.
    ///
    /// Uses the Warshall's algorithm.
    pub fn transitive_closure(&mut self) {
        assert_eq!(self.num_rows(), self.row_bits);
        for pos in 0 .. self.row_bits {
            let (mut rows0, row, mut rows1) = self.split_at_mut(pos);
            for dst_row in rows0.iter_mut().chain(rows1.iter_mut()) {
                if dst_row[pos] {
                    for (dst, src) in dst_row.iter_mut().zip(row.iter()) {
                        *dst |= *src;
                    }
                }
            }
        }
    }
}

impl<'a> BitSubMatrix<'a> {
    /// Iterates over the matrix's rows in the form of mutable slices.
    pub fn iter_mut(&mut self) -> Map<slice::Chunks<Block>,
                                      fn(&[Block]) -> &BitVecSlice> {
        fn f(arg: &[Block]) -> &BitVecSlice {
            unsafe { mem::transmute(arg) }
        }
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        self.slice.chunks(row_size).map(f)
    }
}

impl<'a> BitSubMatrixMut<'a> {
    /// Iterates over the matrix's rows in the form of mutable slices.
    pub fn iter_mut(&mut self) -> Map<slice::ChunksMut<Block>,
                                      fn(&mut [Block]) -> &mut BitVecSlice> {
        fn f(arg: &mut [Block]) -> &mut BitVecSlice {
            unsafe { mem::transmute(arg) }
        }
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        self.slice.chunks_mut(row_size).map(f)
    }
}



// impl BitVecSlice {
//     #[inline]
//     pub fn iter(&self) -> Iter {
//         Iter { bit_slice: self, range: 0..self.length }
//     }
// }

/// Returns `true` if a bit is enabled in the bit vector, or `false` otherwise.
impl Index<usize> for FixedBitVec {
    type Output = bool;

    #[inline]
    fn index(&self, bit: usize) -> &bool {
        if self.get(bit).unwrap() {
            &TRUE
        } else {
            &FALSE
        }
    }
}

/// Returns `true` if a bit is enabled in the matrix, or `false` otherwise.
impl Index<(usize, usize)> for FixedBitMatrix {
    type Output = bool;

    #[inline]
    fn index(&self, (row, col): (usize, usize)) -> &bool {
        let row_size_in_bits = round_up_to_next(self.row_bits, BITS);
        if self.bit_vec.get(row * row_size_in_bits + col).unwrap() {
            &TRUE
        } else {
            &FALSE
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

/// Returns the matrix's row in the form of an immutable slice.
impl Index<usize> for FixedBitMatrix {
    type Output = BitVecSlice;

    #[inline]
    fn index(&self, row: usize) -> &BitVecSlice {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        unsafe {
            mem::transmute(
                &self.bit_vec.storage()[row * row_size .. (row + 1) * row_size]
            )
        }
    }
}

/// Returns the matrix's row in the form of a mutable slice.
impl IndexMut<usize> for FixedBitMatrix {
    #[inline]
    fn index_mut(&mut self, row: usize) -> &mut BitVecSlice {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        unsafe {
            mem::transmute(
                &mut self.bit_vec.storage_mut()[row * row_size .. (row + 1) * row_size]
            )
        }
    }
}

/// Returns `true` if a bit is enabled in the matrix, or `false` otherwise.
impl Index<(usize, usize)> for BitMatrix {
    type Output = bool;

    #[inline]
    fn index(&self, (row, col): (usize, usize)) -> &bool {
        let row_size_in_bits = round_up_to_next(self.row_bits, BITS);
        if self.bit_vec.get(row * row_size_in_bits + col).unwrap_or(false) {
            &TRUE
        } else {
            &FALSE
        }
    }
}

/// Returns the matrix's row in the form of an immutable slice.
impl Index<usize> for BitMatrix {
    type Output = BitVecSlice;

    #[inline]
    fn index(&self, row: usize) -> &BitVecSlice {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        unsafe {
            mem::transmute(
                &self.bit_vec.storage()[row * row_size .. (row + 1) * row_size]
            )
        }
    }
}

/// Returns the matrix's row in the form of a mutable slice.
impl IndexMut<usize> for BitMatrix {
    #[inline]
    fn index_mut(&mut self, row: usize) -> &mut BitVecSlice {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        unsafe {
            mem::transmute(
                &mut self.bit_vec.storage_mut()[row * row_size .. (row + 1) * row_size]
            )
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

impl Deref for FixedBitVec {
    type Target = [Block];

    #[inline]
    fn deref(&self) -> &[Block] {
        &self.data[..]
    }
}

impl DerefMut for FixedBitVec {
    #[inline]
    fn deref_mut(&mut self) -> &mut [Block] {
        &mut self.data[..]
    }
}

/// An iterator for `FixedBitVec`.
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

impl fmt::Debug for FixedBitVec {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for bit in self.iter() {
            try!(write!(fmt, "{}", if bit { 1 } else { 0 }));
        }
        Ok(())
    }
}

// Arithmetic functions

#[inline]
fn div_rem(num: usize, divisor: usize) -> (usize, usize) {
    (num / divisor, num % divisor)
}

#[inline]
 fn round_up_to_next(unrounded: usize, target_alignment: usize) -> usize {
    assert!(target_alignment.is_power_of_two());
    (unrounded + target_alignment - 1) & !(target_alignment - 1)
}

// Tests

#[test]
fn test_0_elements() {
    let vec = FixedBitVec::new();
    assert_eq!(vec.storage().len(), 0);
    assert_eq!(vec.len(), 0);
}

#[test]
fn test_1_element() {
    let mut vec = BitVec::from_elem(1, true);
    assert!(vec[0]);
    assert_eq!(vec.len(), 1);

    let mut expected = BitVec::from_elem(1, false);
    expected.set(0, true);
    assert_eq!(vec, expected);
    vec.clear();
    expected.clear();
    assert_eq!(vec, expected);
}
