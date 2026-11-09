// A generic fn returns a struct holding a moved-in closure.
struct H<F: Fn() -> u8> { f: F }
fn wrap<F: Fn() -> u8>(f: F) -> H<F> { H { f } }
fn main() { let s = String::from("abc"); let h = wrap(move || s.len() as u8); println!("{}", (h.f)()); }
