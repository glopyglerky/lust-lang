// expect: L0306
// rustc E0597: the parameter is the enclosing fn's `'a`; a local passed to
// it does not live that long.
fn f<'a>(keep: &mut Vec<&'a String>) {
    let mut push = |x: &'a String| keep.push(x);
    let local = String::from("l");
    push(&local);
}
fn main() { let mut v = Vec::new(); f(&mut v); println!("{}", v.len()); }
