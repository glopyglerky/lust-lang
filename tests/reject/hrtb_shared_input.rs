// expect: L0203
// Valid Rust: one binder lifetime on two inputs. lust reads `for<..>` only
// where it means what elision means (W-BORROW3, parser/hrtb.rs).
fn ap<F: for<'a> Fn(&'a u8, &'a u8) -> u8>(f: F) -> u8 { f(&1, &2) }
fn main() { println!("{}", ap(|a, b| a + b)); }
