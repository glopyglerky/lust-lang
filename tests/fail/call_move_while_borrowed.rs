// expect: L0311
fn take(s: String) -> usize {
    s.len()
}

fn main() {
    let s = String::from("hi");
    let r = &s;
    let n = take(s);
    println!("{} {}", r.len(), n);
}
