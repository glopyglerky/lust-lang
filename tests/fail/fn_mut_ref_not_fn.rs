// expect: L0612
// rustc E0277: `&mut F` is not `Fn` (core has no such impl).
fn take<F: Fn()>(f: F) { f() }
fn pass<F: FnMut()>(mut f: F) { take(&mut f) }
fn main() { pass(|| println!("hi")); }
