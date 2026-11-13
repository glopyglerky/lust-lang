// core panics on a radix outside 2..=36 before it parses (R26).
fn main() {
    let r = std::env::args().count() as u32;
    println!("{:?}", u8::from_str_radix("1", 16 + r));
    println!("{:?}", i16::from_str_radix("1", r + 36));
}
