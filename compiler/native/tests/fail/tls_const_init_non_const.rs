// expect: L0631
// The `const { .. }` form is a const item in rustc: `String::from` is not
// a const fn (E0015).
thread_local! {
    static S: String = const { String::from("x") };
}
fn main() {
    S.with(|s| println!("{}", s));
}
