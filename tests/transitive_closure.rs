extern crate bit_matrix;

use bit_matrix::FixedBitMatrix;

#[test]
fn test_transitive_closure() {
    let mut matrix = FixedBitMatrix::new(4, 4);
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

    let mut expected_matrix = FixedBitMatrix::new(4, 4);
    for i in 0..4 {
        for j in 0..4 {
            expected_matrix.set(i, j, true);
        }
    }

    assert_eq!(matrix, expected_matrix);
}
