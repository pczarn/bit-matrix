<div align="center">
  <h1>bit-matrix</h1>
  <p>
    <strong>A compact matrix of bits.</strong>
  </p>
  <p>

[![crates.io][crates.io shield]][crates.io link]
[![Documentation][docs.rs badge]][docs.rs link]
![Rust CI][github ci badge]
![MSRV][rustc 1.82+]
<br />
<br />
[![Dependency Status][deps.rs status]][deps.rs link]
[![Download Status][shields.io download count]][crates.io link]

  </p>
</div>

[crates.io shield]: https://img.shields.io/crates/v/bit-matrix?label=latest
[crates.io link]: https://crates.io/crates/bit-matrix
[docs.rs badge]: https://docs.rs/bit-matrix/badge.svg?version=0.9.1
[docs.rs link]: https://docs.rs/bit-matrix/0.9.1/bit-matrix/
[github ci badge]: https://github.com/pczarn/bit-matrix/workflows/CI/badge.svg?branch=master
[rustc 1.82+]: https://img.shields.io/badge/rustc-1.82%2B-blue.svg
[deps.rs status]: https://deps.rs/crate/bit-matrix/0.9.1/status.svg
[deps.rs link]: https://deps.rs/crate/bit-matrix/0.9.1
[shields.io download count]: https://img.shields.io/crates/d/bit-matrix.svg

Rust library that implements bit matrices.
[You can check the documentation here](https://docs.rs/bit-matrix/latest/bit_matrix/).

Built on top of [contain-rs/bit-vec](https://github.com/contain-rs/bit-vec/).


## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
bit-matrix = "0.9"
```

If you want [serde](https://github.com/serde-rs/serde) support, include the feature like this:

```toml
[dependencies]
bit-matrix = { version = "0.9", features = ["serde"] }
```

If you want to use bit-matrix in a program that has `#![no_std]`, just drop default features:

```toml
[dependencies]
bit-matrix = { version = "0.9", default-features = false }
```

If you want to use serde with the alloc crate instead of std, just use the `serde_no_std` feature:

```toml
[dependencies]
bit-matrix = { version = "0.9", default-features = false, features = ["serde", "serde_no_std"] }

## Examples

This simple example calculates the transitive closure of 4x4 bit matrix.

```rust
use bit_matrix::BitMatrix;

fn main() {
    let mut matrix = BitMatrix::new(4, 4);
    let points = &[
        (0, 0),
        (0, 1),
        (0, 3),
        (1, 0),
        (1, 2),
        (2, 0),
        (2, 1),
        (3, 1),
        (3, 3),
    ];
    for &(i, j) in points {
        matrix.set(i, j, true);
    }
    matrix.transitive_closure();

    let mut expected_matrix = BitMatrix::new(4, 4);
    for i in 0..4 {
        for j in 0..4 {
            expected_matrix.set(i, j, true);
        }
    }

    assert_eq!(matrix, expected_matrix);
}
```

## License

Dual-licensed for compatibility with the Rust project.

Licensed under the Apache License Version 2.0:
http://www.apache.org/licenses/LICENSE-2.0, or the MIT license:
http://opensource.org/licenses/MIT, at your option.
