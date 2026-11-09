// expect: L0110
// rustc E0308: upstream `AtFlags` bits are `u32`.
fn main() {
    let f = rustix::fs::AtFlags::from_bits_retain(0x100i32);
    println!("{}", f.bits());
}
