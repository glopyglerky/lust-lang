// expect: L0612
// rustc E0631: `str::cmp` takes `&str`, and `sort_by` over `&str`
// items passes `&&str`. A fn item's parameter is no coercion site.
fn main() {
    let mut v = vec!["b", "a"];
    v.sort_by(str::cmp);
    println!("{:?}", v);
}
