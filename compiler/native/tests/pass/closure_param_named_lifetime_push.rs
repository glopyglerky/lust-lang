// `|x: &'a String|` takes the enclosing fn's `'a`: it may go into a
// captured `Vec<&'a String>`.
fn f<'a>(keep: &mut Vec<&'a String>, s: &'a String) {
    let mut push = |x: &'a String| keep.push(x);
    push(s);
    push(s);
}
fn main() { let s = String::from("s"); let mut v = Vec::new(); f(&mut v, &s); println!("{}", v.len()); }
