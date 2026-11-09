// expect: L0110
// rustc E0271: the Error of `TryFrom<Vec<u8>> for [u8; 2]` is `Vec<u8>`.
fn main() {
    let v = vec![1u8, 2];
    let a: Result<[u8; 2], std::array::TryFromSliceError> = v.try_into();
    println!("{:?}", a.is_ok());
}
