//! Matrix of bits.

use core::cmp;
use core::ops::Range;
use core::ops::{Index, IndexMut};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "memusage")]
use memusage::MemoryReport;

use bit_vec::BitVec;

use super::{FALSE, TRUE};
use crate::local_prelude::*;
use crate::row::Iter;
use crate::util::round_up_to_next;

/// A matrix of bits.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    /// Sets the value of all bits.
    #[inline]
    pub fn set_all(&mut self, enabled: bool) {
        if enabled {
            self.bit_vec.set_all();
        } else {
            self.bit_vec.clear();
        }
    }

    /// Grows the matrix in-place, adding `num_rows` rows filled with `value`.
    pub fn grow(&mut self, num_rows: usize, value: bool) {
        self.bit_vec
            .grow(round_up_to_next(self.row_bits, BITS) * num_rows, value);
    }

    /// Truncates the matrix.
    pub fn truncate(&mut self, num_rows: usize) {
        self.bit_vec
            .truncate(round_up_to_next(self.row_bits, BITS) * num_rows);
    }

    /// Returns a slice of the matrix's rows.
    #[inline]
    pub fn sub_matrix(&self, range: Range<usize>) -> BitSubMatrix {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        BitSubMatrix::new(
            &self.bit_vec.storage()[range.start * row_size..range.end * row_size],
            self.row_bits,
        )
    }

    /// Given a row's index, returns a slice of all rows above that row, a reference to said row,
    /// and a slice of all rows below.
    ///
    /// Functionally equivalent to `(self.sub_matrix(0..row), &self[row],
    /// self.sub_matrix(row..self.num_rows()))`.
    #[inline]
    pub fn split_at(&self, row: usize) -> (BitSubMatrix, BitSubMatrix) {
        (
            self.sub_matrix(0..row),
            self.sub_matrix(row..self.num_rows()),
        )
    }

    /// Given a row's index, returns a slice of all rows above that row, a reference to said row,
    /// and a slice of all rows below.
    #[inline]
    pub fn split_at_mut(&mut self, row: usize) -> (BitSubMatrixMut, BitSubMatrixMut) {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        let (first, second) = unsafe {
            self.bit_vec.storage_mut().split_at_mut(row * row_size)
        };
        (BitSubMatrixMut::new(first, self.row_bits), BitSubMatrixMut::new(second, self.row_bits))
    }

    /// Iterate over bits in the specified row.
    pub fn iter_row(&self, row: usize) -> Iter {
        BitSlice::new(&self[row].slice).iter_bits(self.row_bits)
    }

    /// Computes the transitive closure of the binary relation represented by the matrix.
    ///
    /// Uses the Warshall's algorithm.
    pub fn transitive_closure(&mut self) {
        assert_eq!(self.num_rows(), self.row_bits);
        for pos in 0..self.row_bits {
            let (mut rows0, mut rows1a) = self.split_at_mut(pos);
            let (row, mut rows1b) = rows1a.split_at_mut(1);
            for dst_row in rows0.iter_mut().chain(rows1b.iter_mut()) {
                if dst_row[pos] {
                    for (dst, src) in dst_row.iter_blocks_mut().zip(row[0].iter_blocks()) {
                        *dst |= *src;
                    }
                }
            }
        }
    }

    /// Computes the reflexive closure of the binary relation represented by the matrix.
    pub fn reflexive_closure(&mut self) {
        for i in 0..cmp::min(self.row_bits, self.num_rows()) {
            self.set(i, i, true);
        }
    }
}

/// Returns the matrix's row in the form of an immutable slice.
impl Index<usize> for BitMatrix {
    type Output = BitSlice;

    #[inline]
    fn index(&self, row: usize) -> &BitSlice {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        BitSlice::new(&self.bit_vec.storage()[row * row_size..(row + 1) * row_size])
    }
}

/// Returns the matrix's row in the form of a mutable slice.
impl IndexMut<usize> for BitMatrix {
    #[inline]
    fn index_mut(&mut self, row: usize) -> &mut BitSlice {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        unsafe {
            BitSlice::new_mut(&mut self.bit_vec.storage_mut()[row * row_size..(row + 1) * row_size])
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

#[cfg(feature = "memusage")]
impl MemoryReport for BitMatrix {
    fn indirect(&self) -> usize {
        (self.bit_vec.capacity() + 31) / 32 * 4
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
