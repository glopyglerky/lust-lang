// expect: L0203
// rustc E0308: `Some::<T>` has no late-bound lifetime, so it is not as
// general as `for<'a> fn(&'a str) -> Option<&'a str>`.
fn main() {
    let f: fn(&str) -> Option<&str> = Some;
    println!("{:?}", f("a"));
}
