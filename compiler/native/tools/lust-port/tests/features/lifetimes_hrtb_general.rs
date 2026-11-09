// feature: lifetimes.hrtb_general
// form: a `for<'a>` binder that elision cannot say (one binder lifetime on two inputs)
fn ap<F: for<'a> Fn(&'a u8, &'a u8) -> u8>(f: F) -> u8 { f(&1, &2) }
fn main() { println!("{}", ap(|a, b| a + b)); }
