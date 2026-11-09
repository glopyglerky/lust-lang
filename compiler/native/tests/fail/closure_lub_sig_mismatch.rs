// expect: L0110
// rustc E0308: the closures' signatures differ.
fn main() {
    let v = [|x: u8| x + 1, |x: u16| x as u8];
    println!("{}", (v[0])(1));
}
