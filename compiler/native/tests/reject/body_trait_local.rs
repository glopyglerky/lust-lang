// expect: L0820
// Valid Rust. lust does not take trait items in fn bodies yet (ts-rust has none).
fn main() {
    trait Loc { fn l(&self) -> u8; }
    impl Loc for u8 { fn l(&self) -> u8 { *self + 1 } }
    println!("{}", 4u8.l());
}
