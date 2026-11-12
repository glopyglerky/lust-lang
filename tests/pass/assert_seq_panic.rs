// panic: assertion `left == right` failed: sequence mismatch
fn main() {
    let values = [2u8, 5, 8];
    assert_eq!(&values[..2], &[2, 9], "sequence mismatch");
}
