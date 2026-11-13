// expect: L0306
// rustc E0597: an alias's `&'static` binds a `let` annotation that names it (module and
// block-local aliases), so a borrow of a local does not fit.
type Alias = Option<&'static u32>;
fn main() {
    type R = &'static u32;
    let n = 5u32;
    let x: Alias = Some(&n);
    let y: R = &n;
    println!("{:?} {}", x, y);
}
