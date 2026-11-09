// expect: L0733
// rustc: lifetime may not live long enough (`Box<dyn Fn>` is `'static`).
fn mk(s: &String) -> Box<dyn Fn() -> usize> { Box::new(move || s.len()) }
fn main() { let s = String::from("x"); println!("{}", mk(&s)()); }
