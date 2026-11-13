// expect: L0203
// A constructor at a fn pointer type with references is valid Rust when
// the lifetimes are `'static`; lust has no region signature for a
// constructor yet (L0203).
fn main() {
    let f: fn(&'static str) -> Option<&'static str> = Some;
    println!("{:?}", f("a"));
}
