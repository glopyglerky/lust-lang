// expect: L0733
// rustc: borrowed data escapes outside of function. `impl dyn P` is for `dyn P + 'static`,
// and the guard's object lives for `'b` only.
use std::cell::Ref;
trait P { fn id(&self) -> u32; }
struct A(u32);
impl P for A { fn id(&self) -> u32 { self.0 } }
impl dyn P {
    fn hello(&self) -> u32 {
        9
    }
}
fn f<'b>(r: Ref<'b, dyn P>) -> u32 {
    r.hello()
}
fn main() {}
