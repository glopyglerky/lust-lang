// expect: L0110
// rustc E0308 (one type is more general than the other): an unannotated
// closure argument has a free region, so the closure is not a
// higher-ranked `fn(&str) -> &str`.
fn main() {
    let f = |x| x;
    let g: fn(&str) -> &str = f;
    println!("{}", g("w"));
}
