// ARCH 2.8: a `where Self: 'static` method stays in the vtable and is callable on a `'static` dyn value
// (written, or inferred for a local coercion).
trait T { fn f(&self) -> u8; fn h(&self) -> u8 where Self: 'static { 8 + self.f() } }
trait B: T { fn g(&self) -> u8 where Self: 'static; }
struct S { v: u8 }
impl T for S { fn f(&self) -> u8 { self.v } }
impl B for S { fn g(&self) -> u8 { 50 + self.v } }
fn call(d: &'static dyn T) -> u8 { d.h() }
fn call2(d: &(dyn T + 'static)) -> u8 { d.h() + T::h(d) }
fn call3(d: &mut (dyn B + 'static)) -> u8 { d.g() + d.h() + <dyn B as T>::h(d) }
static GS: S = S { v: 4 };
fn main() {
    let s = S { v: 3 };
    let d: &dyn T = &s;
    println!("{} {} {} {}", call(&GS), call2(&s), d.h(), d.f());
    let mut s2 = S { v: 5 };
    println!("{}", call3(&mut s2));
}
