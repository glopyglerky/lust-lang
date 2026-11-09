// A format argument whose type is known only at body end prints through
// the written impl the solver selects, not a field walk.
use std::cell::OnceCell;
use std::fmt;
struct W<T>(T);
impl<T: fmt::Debug> fmt::Debug for W<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "W<{:?}>", self.0)
    }
}
impl<T> From<T> for W<T> {
    fn from(t: T) -> Self {
        W(t)
    }
}
fn main() {
    let c = W::from(3);
    println!("{:?}", c);
    println!("{:?} {:#?}", OnceCell::from(5u8), OnceCell::from(String::from("a")));
    let mut y = None;
    println!("{:?}", y);
    y = Some(OnceCell::from(W::from('x')));
    println!("{:?}", y);
}
