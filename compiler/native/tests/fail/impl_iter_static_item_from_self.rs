// expect: L0733
// rustc: lifetime may not live long enough. `Item = &'static str` but
// the items borrow `self`.
struct S {
    v: Vec<String>,
}
impl S {
    fn f(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.v.iter().map(|x| x.as_str())
    }
}
fn main() {
    let s = S { v: vec!["x".into()] };
    for x in s.f() {
        println!("{}", x);
    }
}
