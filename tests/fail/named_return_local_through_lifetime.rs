// expect: L0307
// rustc E0515: returning a borrow of a local through a named lifetime.
fn make<'a>(_x: &'a str) -> &'a str {
    let s = String::from("local");
    &s
}

fn main() {
    println!("{}", make("a"));
}
