// expect: L0733
// rustc: lifetime may not live long enough; a `&dyn Tr` param is not `dyn Tr + 'static`.
trait Tr { fn f(&self) -> u8; }
fn up(d: &dyn Tr) -> &(dyn Tr + 'static) { d }
fn main() {}
