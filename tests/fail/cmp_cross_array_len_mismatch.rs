// expect: L0612
// rustc E0277: arrays of different lengths have no `PartialEq` impl.
fn main() {
    let a = [1u8, 2];
    println!("{}", a.eq(&[1u8, 2, 3]));
}
