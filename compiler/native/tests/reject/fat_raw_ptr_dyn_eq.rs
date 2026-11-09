// expect: L0112
// Valid Rust: == of two dyn raw pointers compares the vtable addresses, which rustc does not fix (one vtable per codegen unit, identical ones merged), so lust rejects it (tf-fatptr). ptr::addr_eq is supported.
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
    let a: std::rc::Rc<dyn Tr> = std::rc::Rc::new(S(1));
    let p = std::rc::Rc::as_ptr(&a);
    println!("{} {}", p == p, a.f());
}
