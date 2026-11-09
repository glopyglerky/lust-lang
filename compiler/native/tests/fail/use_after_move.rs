// expect: L0301
fn take(s: String) -> usize {
    s.len()
}

fn main() {
    let s = String::from("x");
    take(s);
    println!("{}", s);
}
