// expect: L0110
// rustc E0308: on `&i32`, `max` is `Ord::max` with Self = `&i32`, so the
// argument must be a reference too.
fn main() {
    let a = &3;
    println!("{}", a.max(5));
}
