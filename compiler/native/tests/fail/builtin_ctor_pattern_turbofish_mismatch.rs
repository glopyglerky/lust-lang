// expect: L0110
// rustc E0308: the pattern's turbofish fixes the element type.
fn main() {
    let o: Option<u16> = Some(3);
    match o {
        Some::<u8>(x) => println!("{}", x),
        None => println!("none"),
    }
}
