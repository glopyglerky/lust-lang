// expect: L0110
// rustc E0525: a call through a Box takes `Fn`; this closure is only FnOnce.
fn main() {
    let s = String::from("x");
    let b = Box::new(move || s);
    println!("{}", b());
}
