// expect: L0301
// rustc: borrow of moved value. The `@` binding moves the value its
// subpattern borrows; rustc rejects this even when nothing is used.
fn f(x: Option<String>) -> usize {
    match x {
        a @ Some(ref b) => 0,
        _ => 1,
    }
}
fn main() {
    println!("{}", f(Some(String::from("hi"))));
}
