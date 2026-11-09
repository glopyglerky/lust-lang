// expect: L0110
// rustc E0308: `Ok(7u16)` is not a `Result<u8, &str>`.
static B: Result<u8, &str> = Ok(7u16);
fn main() {
    println!("{:?}", B);
}
