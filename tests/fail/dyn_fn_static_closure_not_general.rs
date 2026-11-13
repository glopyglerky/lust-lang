// expect: L0735
// rustc: implementation of `Fn` is not general enough. The closure takes only `&'static str`; `dyn Fn(&str)` passes any.
fn main() {
    let c = |s: &'static str| s.len();
    let f: &dyn Fn(&str) -> usize = &c;
    println!("{}", f("abc"));
}
