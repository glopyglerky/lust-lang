// expect: L0110
fn main() {
    let values = [2u8, 5];
    assert_eq!(&values[..], &[2u16, 5]);
}
