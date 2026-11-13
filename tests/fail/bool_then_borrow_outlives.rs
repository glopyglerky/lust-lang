// expect: L0733
// rustc "lifetime may not live long enough": the closure's result
// borrows `self`, but the return type says 'static.
struct H { p: String }
impl H { fn f(&self) -> Option<&'static str> { true.then(|| self.p.as_str()) } }
fn main() { let h = H { p: String::from("a") }; println!("{:?}", h.f()); }
