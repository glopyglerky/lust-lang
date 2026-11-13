// expect: L0540
fn main() {
    println!("{}", u8::MAX as f32 as u8 + 1);
}
