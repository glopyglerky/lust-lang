// expect: L0612
// rustc E0308: `Box<i32>` has one `PartialEq` impl, `Rhs = Box<i32>`,
// so the argument must be a `&Box<i32>`; `&5` is not.
fn main() {
    let b = Box::new(5i32);
    println!("{}", b.eq(&5));
}
