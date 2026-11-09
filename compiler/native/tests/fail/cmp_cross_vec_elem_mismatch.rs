// expect: L0612
// rustc E0277: `Vec<u8>: PartialEq<[i64; 2]>` needs `u8: PartialEq<i64>`.
fn main() {
    let v: Vec<u8> = vec![1, 2];
    println!("{}", v.eq(&[1i64, 2]));
}
