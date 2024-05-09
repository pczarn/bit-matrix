extern crate bit_matrix;
#[cfg(feature = "serde")]
extern crate serde;
#[cfg(feature = "serde")]
extern crate serde_json;

#[cfg(feature = "serde")]
#[test]
fn test_serialize_deserialize() {
    use bit_matrix::BitMatrix;

    let mut expected_matrix = BitMatrix::new(4, 4);
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
        expected_matrix.set(i, j, true);
    }

    let serialized = serde_json::to_string(&expected_matrix).unwrap();
    let matrix: BitMatrix = serde_json::from_str(serialized.as_str()).unwrap();

    assert_eq!(matrix, expected_matrix);
}
