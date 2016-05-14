//! Submatrix of bits.

use std::slice;
use std::fmt;
use std::iter::Map;
use std::mem;
use std::ops::{Index, IndexMut};
use std::ops::Range;

use super::{Block, BITS};
use row::BitVecSlice;
use util::{div_rem, round_up_to_next};

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

impl<'a> BitSubMatrix<'a> {
    /// Returns a new BitSubMatrix.
    pub fn new(slice: &[Block], row_bits: usize) -> BitSubMatrix {
        BitSubMatrix {
            slice: slice,
            row_bits: row_bits,
        }
    }

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
    /// Returns a new BitSubMatrixMut.
    pub fn new(slice: &mut [Block], row_bits: usize) -> BitSubMatrixMut {
        BitSubMatrixMut {
            slice: slice,
            row_bits: row_bits,
        }
    }

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
