// expect: L0306
// rustc E0597: `type A = Q<'static>` makes the struct's lifetime `'static` in a `let`
// annotation that names the alias, so a borrow of a local does not fit.
struct Q<'q> {
    r: &'q u32,
}
type A = Q<'static>;
fn main() {
    let n = 5u32;
    let x: A = Q { r: &n };
    println!("{}", x.r);
}
