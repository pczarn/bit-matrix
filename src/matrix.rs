//! Matrix of bits.

use std::mem;
use std::ops::{Index, IndexMut};
use std::ops::Range;

use bit_vec::BitVec;

use row::{BitVecSlice, Iter};
use submatrix::{BitSubMatrix, BitSubMatrixMut};
use util::round_up_to_next;
use super::{BITS, TRUE, FALSE};

/// A matrix of bits.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitMatrix {
    bit_vec: BitVec,
    row_bits: usize,
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
        BitSubMatrix::new(
            &self.bit_vec.storage()[range.start * row_size .. range.end * row_size],
            self.row_bits
        )
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
                        BitSubMatrix) {
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
        BitVecSlice::new(&self[row]).iter_bits(self.row_bits)
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
