fn square(x: i64) -> i64 {
    x * x
}

#[test]
fn test_square() {
    assert_eq!(square(4), 16);
}

#[test]
fn test_square_zero() {
    assert_eq!(square(0), 0);
}
