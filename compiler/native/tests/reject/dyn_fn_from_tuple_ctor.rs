// expect: L0702
// Valid Rust: a tuple-struct constructor as a `dyn Fn` value. lust has no
// fn body for a constructor's vtable entry yet.
struct W(u8);
fn main() { let f: Box<dyn Fn(u8) -> W> = Box::new(W); println!("{}", f(3).0); }
