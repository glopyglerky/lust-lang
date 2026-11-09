// expect: L0203
// rustc accepts; lust cannot tie a named closure parameter to the
// enclosing fn's lifetime when another closure calls it (L0203).
fn total<'a>(v: &'a Vec<String>) -> usize {
    let len = |x: &'a String| x.len();
    v.iter().map(|s| len(s)).sum()
}
fn main() { let v = vec![String::from("ab")]; println!("{}", total(&v)); }
