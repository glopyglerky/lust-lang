// expect: L0602
// Valid Rust: ptr::eq of dyn pointers compares vtable addresses, which rustc does not fix; lust rejects it (tf-fatptr).
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
    println!("{} {}", std::ptr::eq(p, p), a.f());
}
