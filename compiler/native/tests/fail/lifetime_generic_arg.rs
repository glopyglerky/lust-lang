// expect: L0203
fn pick(x: &'a str) -> usize { x.len() }
fn main() { println!("{}", pick("hi")); }
