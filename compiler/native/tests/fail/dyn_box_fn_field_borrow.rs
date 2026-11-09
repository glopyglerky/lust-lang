// expect: L0306
// rustc E0597: a `Box<dyn Fn>` field is `'static`, the closure borrows `s`.
struct H { f: Box<dyn Fn() -> usize> }
fn main() {
    let h;
    { let s = String::from("x"); h = H { f: Box::new(|| s.len()) }; }
    println!("{}", (h.f)());
}
