// expect: L0306
// rustc E0597: the parameter wants `'static`; a local borrow is shorter.
fn keep(x: &'static str) -> usize {
    x.len()
}

fn main() {
    let s = String::from("hi");
    println!("{}", keep(&s));
}
