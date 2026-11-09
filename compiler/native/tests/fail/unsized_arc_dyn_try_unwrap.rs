// expect: L0110
// rustc E0277: `Arc::try_unwrap` needs `T: Sized`; `dyn T` is not.
use std::sync::Arc;
trait T { fn f(&self) -> u8; }
struct S;
impl T for S { fn f(&self) -> u8 { 1 } }
fn main() { let a: Arc<dyn T> = Arc::new(S); let r = Arc::try_unwrap(a); println!("{}", r.is_ok()); }
