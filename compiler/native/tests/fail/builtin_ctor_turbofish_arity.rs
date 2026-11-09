// expect: L0107
// rustc E0107: Option takes one generic argument.
fn main() {
    let o = Some::<u8, u8>(3);
    println!("{:?}", o);
}
