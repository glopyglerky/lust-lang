// expect: L0612
// rustc E0277: no `TryFrom<u8>` for `bool` from `f64` either way;
// here in method form with the target named by the binding.
fn main() {
    let x = 1.5f64;
    let r: Result<u8, String> = x.try_into();
    println!("{:?}", r);
}
