// expect: L0112
// Valid Rust: == on *const T for T: ?Sized could compare dyn vtable addresses at an instance, so lust needs T: Sized there (tf-fatptr).
trait Tr {
    fn f(&self) -> u32;
}
struct S(u32);
impl Tr for S {
    fn f(&self) -> u32 {
        self.0
    }
}
fn main() {
    fn same<T: ?Sized>(a: *const T, b: *const T) -> bool {
        a == b
    }
    let v = [1, 2];
    println!("{}", same(&v[..] as *const [i32], &v[..]));
}
