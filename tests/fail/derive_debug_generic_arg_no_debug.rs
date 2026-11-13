// expect: L0112
// rustc E0277: `No` is not Debug, so `W<&No>` is not Debug under
// the conditional `impl<K: Debug> Debug for W<K>`.
use std::fmt;
pub struct W<K>(K);
impl<K: fmt::Debug> fmt::Debug for W<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}
struct No;
#[derive(Debug)]
struct A { m: W<&'static No> }
fn main() { println!("x"); }
