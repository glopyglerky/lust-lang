// expect: L0612
// rustc E0277: a `fn(&char) -> bool` is not `FnMut(char) -> bool`, so no Pattern.
fn ws(c: &char) -> bool { *c == ' ' }
fn main() {
    let f: fn(&char) -> bool = ws;
    println!("{}", "a b ".trim_end_matches(f));
}
