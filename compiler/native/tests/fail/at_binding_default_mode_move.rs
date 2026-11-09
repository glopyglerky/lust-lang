// expect: L0301
// rustc: borrow of moved value. Default binding modes make `a` move the
// `&mut` and `n` a `ref mut` binding.
fn main() { let mut x = Some((1, String::new())); match &mut x { a @ Some((n, _)) => {}, _ => {} } }
