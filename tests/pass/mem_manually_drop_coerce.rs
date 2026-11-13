// Deref coercion through ManuallyDrop and then String: `&m` as `&str`.
use std::mem::ManuallyDrop;
fn show(s: &str) -> usize {
    s.len()
}
fn main() {
    let m = ManuallyDrop::new(String::from("hi"));
    let s: &str = &m;
    println!("{s} {}", show(&m));
    drop(ManuallyDrop::into_inner(m));
}
