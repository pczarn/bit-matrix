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
        if self.row_bits == 0 {
            0
        } else {
            let row_blocks = round_up_to_next(self.row_bits, BITS) / BITS;
            self.bit_vec.storage().len() / row_blocks
        }
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

    /// Grows the matrix in-place, adding `num_rows` rows filled with `value`.
    pub fn grow(&mut self, num_rows: usize, value: bool) {
        self.bit_vec.grow(round_up_to_next(self.row_bits, BITS) * num_rows, value);
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
    /// Forms a BitSubMatrix from a pointer and dimensions.
    pub unsafe fn from_raw_parts(ptr: *const Block, rows: usize, row_bits: usize) -> Self {
        BitSubMatrix {
            slice: slice::from_raw_parts(ptr, round_up_to_next(row_bits, BITS) / BITS * rows),
            row_bits: row_bits,
        }
    }

    /// Iterates over the matrix's rows in the form of mutable slices.
    pub fn iter(&self) -> Map<slice::Chunks<Block>,
                                      fn(&[Block]) -> &BitVecSlice> {
        fn f(arg: &[Block]) -> &BitVecSlice {
            unsafe { mem::transmute(arg) }
        }
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        self.slice.chunks(row_size).map(f)
    }
}

impl<'a> BitSubMatrixMut<'a> {
    /// Forms a BitSubMatrix from a pointer and dimensions.
    pub unsafe fn from_raw_parts(ptr: *mut Block, rows: usize, row_bits: usize) -> Self {
        BitSubMatrixMut {
            slice: slice::from_raw_parts_mut(ptr, round_up_to_next(row_bits, BITS) / BITS * rows),
            row_bits: row_bits,
        }
    }

    /// Returns the number of rows.
    #[inline]
    fn num_rows(&self) -> usize {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        self.slice.len() / row_size
    }

    /// Sets the value of a bit.
    ///
    /// # Panics
    ///
    /// Panics if `(row, col)` is out of bounds.
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, enabled: bool) {
        let row_size_in_bits = round_up_to_next(self.row_bits, BITS);
        let bit = row * row_size_in_bits + col;
        let (block, i) = div_rem(bit, BITS);
        assert!(block < self.slice.len() && col < self.row_bits);
        unsafe {
            let elt = self.slice.get_unchecked_mut(block);
            if enabled {
                *elt |= 1 << i;
            } else {
                *elt &= !(1 << i);
            }
        }
    }

    /// Returns a slice of the matrix's rows.
    #[inline]
    pub fn sub_matrix(&self, range: Range<usize>) -> BitSubMatrix {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        BitSubMatrix {
            slice: &self.slice[range.start * row_size .. range.end * row_size],
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



impl BitVecSlice {
    /// Iterates over bits.
    #[inline]
    pub fn iter_bits(&self, len: usize) -> Iter {
        Iter { bit_slice: self, range: 0..len }
    }
}

/// Returns `true` if a bit is enabled in the matrix, or `false` otherwise.
impl Index<(usize, usize)> for BitMatrix {
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

/// Returns the matrix's row in the form of a mutable slice.
impl<'a> Index<usize> for BitSubMatrixMut<'a> {
    type Output = BitVecSlice;

    #[inline]
    fn index(&self, row: usize) -> &BitVecSlice {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        unsafe {
            mem::transmute(
                &self.slice[row * row_size .. (row + 1) * row_size]
            )
        }
    }
}

/// Returns the matrix's row in the form of a mutable slice.
impl<'a> IndexMut<usize> for BitSubMatrixMut<'a> {
    #[inline]
    fn index_mut(&mut self, row: usize) -> &mut BitVecSlice {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        unsafe {
            mem::transmute(
                &mut self.slice[row * row_size .. (row + 1) * row_size]
            )
        }
    }
}

/// Returns the matrix's row in the form of a mutable slice.
impl<'a> Index<usize> for BitSubMatrix<'a> {
    type Output = BitVecSlice;

    #[inline]
    fn index(&self, row: usize) -> &BitVecSlice {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        unsafe {
            mem::transmute(
                &self.slice[row * row_size .. (row + 1) * row_size]
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

impl<'a> fmt::Debug for BitSubMatrix<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for row in self.iter() {
            for bit in row.iter_bits(self.row_bits) {
                try!(write!(fmt, "{}", if bit { 1 } else { 0 }));
            }
            try!(write!(fmt, "\n"));
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
fn test_empty() {
    let mut matrix = BitMatrix::new(0, 0);
    for _ in 0..3 {
        assert_eq!(matrix.num_rows(), 0);
        assert_eq!(matrix.size(), (0, 0));
        matrix.transitive_closure();
    }
}
