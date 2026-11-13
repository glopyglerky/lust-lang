// expect: L0307
// rustc E0515: only the `&[]` expression is promoted; a borrow of a local holding `[]` is not.
fn f() -> &'static [u8] {
    let a = [];
    &a
}
fn main() {
    println!("{}", f().len());
}
