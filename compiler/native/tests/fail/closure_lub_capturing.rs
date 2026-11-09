// expect: L0110
// rustc E0308: only closures that capture nothing join as a fn pointer.
fn main() {
    let x = 5;
    let c = move || x;
    let d = || 2;
    let v = if true { d } else { c };
    println!("{}", v());
}
