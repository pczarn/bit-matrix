//! Submatrix of bits.

use core::cmp;
use core::fmt;
use core::mem;
use core::ops::RangeBounds;
use core::ops::{Index, IndexMut};
use core::slice;

use crate::local_prelude::*;
use crate::util::{div_rem, round_up_to_next};

/// Immutable access to a range of matrix's rows.
pub struct BitSubMatrix<'a> {
    pub(crate) slice: &'a [Block],
    pub(crate) row_bits: usize,
}

/// Mutable access to a range of matrix's rows.
pub struct BitSubMatrixMut<'a> {
    pub(crate) slice: &'a mut [Block],
    pub(crate) row_bits: usize,
}

impl<'a> BitSubMatrix<'a> {
    /// Returns a new BitSubMatrix.
    pub fn new(slice: &[Block], row_bits: usize) -> BitSubMatrix<'_> {
        BitSubMatrix {
            slice: slice,
            row_bits: row_bits,
        }
    }

    /// Forms a BitSubMatrix from a pointer and dimensions.
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *const Block, rows: usize, row_bits: usize) -> Self {
        BitSubMatrix {
            slice: slice::from_raw_parts(ptr, round_up_to_next(row_bits, BITS) / BITS * rows),
            row_bits: row_bits,
        }
    }

    /// Iterates over the matrix's rows in the form of mutable slices.
    pub fn iter(&self) -> impl Iterator<Item = &BitSlice> {
        fn f(arg: &[Block]) -> &BitSlice {
            unsafe { mem::transmute(arg) }
        }
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        self.slice.chunks(row_size).map(f)
    }
}

impl<'a> BitSubMatrixMut<'a> {
    /// Returns a new BitSubMatrixMut.
    pub fn new(slice: &mut [Block], row_bits: usize) -> BitSubMatrixMut<'_> {
        BitSubMatrixMut {
            slice: slice,
            row_bits: row_bits,
        }
    }

    /// Forms a BitSubMatrix from a pointer and dimensions.
    #[inline]
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
        if row_size == 0 {
            0
        } else {
            self.slice.len() / row_size
        }
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
    pub fn sub_matrix<R: RangeBounds<usize>>(&self, range: R) -> BitSubMatrix<'_> {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        BitSubMatrix {
            slice: &self.slice[(
                range.start_bound().map(|&s| s * row_size),
                range.end_bound().map(|&e| e * row_size),
            )],
            row_bits: self.row_bits,
        }
    }

    /// Given a row's index, returns a slice of all rows above that row, a reference to said row,
    /// and a slice of all rows below.
    ///
    /// Functionally equivalent to `(self.sub_matrix(0..row), &self[row],
    /// self.sub_matrix(row..self.num_rows()))`.
    #[inline]
    pub fn split_at(&self, row: usize) -> (BitSubMatrix<'_>, BitSubMatrix<'_>) {
        (
            self.sub_matrix(0..row),
            self.sub_matrix(row..self.num_rows()),
        )
    }

    /// Given a row's index, returns a slice of all rows above that row, a reference to said row,
    /// and a slice of all rows below.
    #[inline]
    pub fn split_at_mut(&mut self, row: usize) -> (BitSubMatrixMut<'_>, BitSubMatrixMut<'_>) {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        let (first, second) = self.slice.split_at_mut(row * row_size);
        (
            BitSubMatrixMut::new(first, self.row_bits),
            BitSubMatrixMut::new(second, self.row_bits),
        )
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
                        *dst |= src;
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

    /// Iterates over the matrix's rows in the form of mutable slices.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut BitSlice> {
        fn f(arg: &mut [Block]) -> &mut BitSlice {
            unsafe { mem::transmute(arg) }
        }
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        self.slice.chunks_mut(row_size).map(f)
    }
}

/// Returns the matrix's row in the form of a mutable slice.
impl<'a> Index<usize> for BitSubMatrixMut<'a> {
    type Output = BitSlice;

    #[inline]
    fn index(&self, row: usize) -> &BitSlice {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        unsafe { mem::transmute(&self.slice[row * row_size..(row + 1) * row_size]) }
    }
}

/// Returns the matrix's row in the form of a mutable slice.
impl<'a> IndexMut<usize> for BitSubMatrixMut<'a> {
    #[inline]
    fn index_mut(&mut self, row: usize) -> &mut BitSlice {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        unsafe { mem::transmute(&mut self.slice[row * row_size..(row + 1) * row_size]) }
    }
}

/// Returns the matrix's row in the form of a mutable slice.
impl<'a> Index<usize> for BitSubMatrix<'a> {
    type Output = BitSlice;

    #[inline]
    fn index(&self, row: usize) -> &BitSlice {
        let row_size = round_up_to_next(self.row_bits, BITS) / BITS;
        unsafe { mem::transmute(&self.slice[row * row_size..(row + 1) * row_size]) }
    }
}

impl<'a> fmt::Debug for BitSubMatrix<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for row in self.iter() {
            for bit in row.iter_bits(self.row_bits) {
                write!(fmt, "{}", if bit { 1 } else { 0 })?;
            }
            write!(fmt, "\n")?;
        }
        Ok(())
    }
}
