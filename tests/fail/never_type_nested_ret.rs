// expect: L0603
// rustc E0658: `!` inside a return type (`Option<!>`) is not the whole
// return type, so it is the unstable never type.
fn f() -> Option<!> {
    None
}
fn main() {
    println!("{}", f().is_none());
}
