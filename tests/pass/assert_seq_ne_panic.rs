// panic: assertion `left != right` failed
fn main() {
    let values = [2u8, 5, 8];
    assert_ne!(&values[..2], &[2, 5]);
}
