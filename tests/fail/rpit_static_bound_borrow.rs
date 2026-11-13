// expect: L0733
// rustc: lifetime may not live long enough. The hidden closure captures `v`, but the bound says `'static` (hunt2 rpit_static_bound_hidden_borrow).
fn bad(v: &String) -> impl Fn() -> usize + 'static {
    let s = String::new();
    move || v.len() + s.len()
}
fn main() {
    let v = String::from("abc");
    let f = bad(&v);
    println!("{}", f());
}
