// expect: L0202
fn pick(a: &str, b: &str) -> &str {
    a
}

fn main() {
    println!("{}", pick("a", "b"));
}
