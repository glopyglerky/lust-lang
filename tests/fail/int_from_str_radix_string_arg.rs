// expect: L0110
// `from_str_radix` takes `&str`, not a `String` by value.
fn main() {
    let s = String::from("ff");
    println!("{:?}", u8::from_str_radix(s, 16));
}
