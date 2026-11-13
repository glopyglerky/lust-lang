// rustc-error: E0107
// lust-expect: L0107
fn f<T>(x: T) -> T { x }
fn main() { println!("{}", f::<u8, u8>(1)); }
