//! Implements bit matrices.

#![deny(missing_docs,
        missing_copy_implementations,
        trivial_casts,
        trivial_numeric_casts,
        unused_import_braces,
        unused_qualifications)]

#![cfg_attr(test, deny(warnings))]

extern crate bit_vec;

pub mod matrix;
pub mod row;
pub mod submatrix;
mod util;

pub use matrix::BitMatrix;

/// A value for borrowing.
pub static TRUE: bool = true;
/// A value for borrowing.
pub static FALSE: bool = false;

/// The number of bits in a block.
pub const BITS: usize = 32;
/// The type for storing bits.
pub type Block = u32;
