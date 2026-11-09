// expect: L0308
// rustc E0596: borrowing a tuple field mutably through Deref needs DerefMut.
struct W<T> { v: T }
impl<T> std::ops::Deref for W<T> { type Target = T; fn deref(&self) -> &T { &self.v } }
struct N(i32);
fn main() {
    let a = W { v: N(1) };
    let r = &mut a.0;
    *r = 3;
}
