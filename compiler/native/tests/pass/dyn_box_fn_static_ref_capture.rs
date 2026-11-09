// The closure captures a `&'static str`, so the `'static` box holds (R02: the borrow checker
// proves the capture's region outlives `'static` at the unsizing).
fn mk(s: &'static str) -> Box<dyn Fn() -> usize> { Box::new(move || s.len()) }
fn main() { println!("{}", mk("abcd")()); }
