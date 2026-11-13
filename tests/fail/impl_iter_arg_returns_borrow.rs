// expect: L0306
// rustc E0597: the item borrows `v`, which dies first.
fn f<'a>(it: impl Iterator<Item = &'a i32>) -> &'a i32 { it.max().unwrap() }
fn main() { let r; { let v = vec![1, 2]; r = f(v.iter()); } println!("{}", r); }
