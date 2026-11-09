// A closure with a named-lifetime parameter, stored and passed to `map`.
fn f<'a>(v: &'a [String]) -> Vec<&'a str> {
    let conv = |s: &'a String| s.as_str();
    v.iter().map(conv).collect()
}
fn main() { let v = vec![String::from("a")]; println!("{:?}", f(&v)); }
