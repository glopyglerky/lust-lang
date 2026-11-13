// expect: L0301
// rustc E0382: an impl Trait value is not Copy unless its bounds say so, so the first
// `let` moves it even though its hidden type `u8` is Copy (lowering moves it too).
fn make() -> impl std::fmt::Display { 5u8 }
fn main() { let a = make(); let b = a; println!("{} {}", a, b); }
