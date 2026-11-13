// expect: L0309
// rustc E0594: assigning to a tuple field through Deref needs DerefMut.
struct W<T> { v: T }
impl<T> std::ops::Deref for W<T> { type Target = T; fn deref(&self) -> &T { &self.v } }
struct N(i32);
fn main() {
    let mut a = W { v: N(1) };
    a.0 = 2;
    println!("{}", a.0);
}
