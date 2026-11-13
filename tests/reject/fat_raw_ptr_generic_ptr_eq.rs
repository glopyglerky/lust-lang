// expect: L0602
// Valid Rust: ptr::eq on *const T for T: ?Sized could compare dyn vtable addresses at an instance, so lust needs T: Sized there (tf-fatptr).
fn same<T: ?Sized>(a: *const T, b: *const T) -> bool {
    std::ptr::eq(a, b)
}
fn main() {
    let v = [1, 2];
    println!("{}", same(&v[..] as *const [i32], &v[..]));
}
