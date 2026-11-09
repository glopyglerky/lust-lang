// feature: lifetimes.static_anon
// form: &'static refs and fields, T: 'static bound, as in ts-rust leaked arenas
const NAME: &'static str = "lust";
struct View { text: &'static str }
fn boxed<T: 'static>(x: T) -> Box<T> { Box::new(x) }
fn main() { let v = View { text: NAME }; println!("{} {}", v.text, *boxed(3)); }
