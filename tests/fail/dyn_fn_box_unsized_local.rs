// expect: L0110
// rustc E0277: `*f` for `f: Box<dyn Fn()>` cannot be a local.
fn main() { let f: Box<dyn Fn() -> u8> = Box::new(|| 1); let g = *f; println!("{}", g()); }
