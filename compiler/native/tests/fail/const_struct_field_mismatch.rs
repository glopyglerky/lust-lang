// expect: L0110
struct P {
    x: u8,
}
const V: P = P { x: 5u32 };
fn main() {
    println!("{}", V.x);
}
