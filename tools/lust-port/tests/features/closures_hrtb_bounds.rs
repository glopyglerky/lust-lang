// feature: closures.hrtb_bounds
// form: Fn(&T) and FnMut(&str) bounds, which make an implicit for<'a> bound
fn call<F: Fn(&i32) -> i32>(f: F) -> i32 { f(&10) + f(&20) }
fn call_mut<F: FnMut(&str) -> usize>(mut f: F) -> usize { f("ab") + f("cde") }
fn main() { println!("{} {}", call(|n| *n + 1), call_mut(|s| s.len())); }
