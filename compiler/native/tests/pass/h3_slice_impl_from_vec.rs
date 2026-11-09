// A user impl for `[T]` or `str` is reached from a Vec or String receiver:
// rustc's autoderef steps through Vec's and String's Deref.
struct S;
trait Tag { fn tag(&self) -> usize; fn bump(&mut self) -> usize { 7 } }
impl Tag for S { fn tag(&self) -> usize { 9 } }
impl<T> Tag for [T] { fn tag(&self) -> usize { self.len() } }
trait St { fn st(&self) -> usize; }
impl St for str { fn st(&self) -> usize { self.len() } }
trait Bv { fn byv(self) -> usize; }
impl<T> Bv for &[T] { fn byv(self) -> usize { self.len() + 10 } }
fn main() {
    let mut v = vec![S, S];
    let s = String::from("abcd");
    println!("{} {} {} {}", v.tag(), (&v).tag(), Box::new(vec![1]).tag(), v.bump());
    println!("{} {} {}", s.st(), (&s).st(), vec![S].byv());
}
