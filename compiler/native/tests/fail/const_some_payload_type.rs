// expect: L0110
// rustc E0308: `Some(5u16)` is not an `Option<u8>` (this was accepted
// and narrowed to `Some(5)`).
static S: Option<u8> = Some(5u16);
fn main() {
    println!("{:?}", S);
}
