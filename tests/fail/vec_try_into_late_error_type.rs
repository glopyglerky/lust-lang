// expect: L0110
// rustc E0271: the Error of `TryFrom<Vec<u8>> for [u8; 2]` is `Vec<u8>`.
fn main() {
    let v = vec![1u8, 2];
    let r = v.try_into();
    let e: std::array::TryFromSliceError = r.unwrap_err();
    let a: [u8; 2] = r.unwrap();
    println!("{:?}", a);
}
