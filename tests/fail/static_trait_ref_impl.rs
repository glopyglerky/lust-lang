// expect: L0612
// rustc: `trait T: 'static` needs a 'static Self type; `&u32` in an impl header has a lifetime parameter.
trait T: 'static { fn f(&self) {} }
impl T for &u32 {}
fn main() {}
