// expect: L0110
// rustc E0308: `fn(&'static str) -> usize` takes only `'static` borrows, so it is not a `fn(&str) -> usize`.
fn takes(s: &'static str) -> usize {
    s.len()
}

fn main() {
    let p: fn(&'static str) -> usize = takes;
    let q: fn(&str) -> usize = p;
    let s = String::from("x");
    println!("{}", q(&s));
}
