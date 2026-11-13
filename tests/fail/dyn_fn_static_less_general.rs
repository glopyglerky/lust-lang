// expect: L0110
// rustc E0308: `dyn Fn(&'static str)` is not as general as `dyn Fn(&str)`.
fn main() {
    let c = |s: &'static str| s.len();
    let d: &dyn Fn(&'static str) -> usize = &c;
    let e: &dyn Fn(&str) -> usize = d;
    let s = String::from("x");
    println!("{}", e(&s));
}
