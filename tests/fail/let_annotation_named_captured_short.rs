// expect: L0306
// rustc E0597: the closure stores a borrow of `local` into a capture annotated `Option<&'a String>`.
fn run<'a>(v: &'a String) -> usize {
    let mut x: Option<&'a String> = None;
    let local = "x".to_string();
    let mut f = || { x = Some(&local); };
    f();
    drop(local);
    x.map_or(0, |s| s.len()) + v.len()
}
fn main() { let s = "abc".to_string(); println!("{}", run(&s)); }
