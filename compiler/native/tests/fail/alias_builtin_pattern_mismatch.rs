// expect: L0110
// rustc E0308: the alias pattern is `Option<u8>`, the scrutinee `Option<u16>`.
type OP = Option<u8>;
fn main() {
    let o: Option<u16> = Some(3);
    match o { OP::Some(x) => println!("{x}"), OP::None => {} }
}
