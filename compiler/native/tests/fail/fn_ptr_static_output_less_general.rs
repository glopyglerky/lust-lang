// expect: L0110
// rustc E0308: `fn(&str) -> &str` returns its argument's lifetime, not `'static`.
fn id(s: &str) -> &str {
    s
}

fn main() {
    let p: fn(&str) -> &str = id;
    let q: fn(&str) -> &'static str = p;
}
