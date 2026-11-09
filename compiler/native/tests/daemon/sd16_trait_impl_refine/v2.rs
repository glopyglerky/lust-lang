use std::mem::size_of_val;
trait Get { fn get(&self) -> &str; fn take<T: Clone>(&self, x: T) -> usize; }
struct A;
struct B(String);
impl Get for A {
    fn get(&self) -> &str { "a" }
    fn take<T: Clone>(&self, x: T) -> usize { size_of_val(&x) }
}
impl Get for B {
    fn get(&self) -> &str { &self.0 }
    fn take<T: Clone>(&self, x: T) -> usize { let _ = x.clone(); 1 }
}
fn keep(a: &'static A) -> &'static str { a.get() }
fn through<G: Get>(g: &G) -> usize { g.get().len() + g.take(1u8) }
struct NoClone;
fn main() {
    let b = B("bb".to_string());
    println!("{} {} {} {}", keep(&A), through(&A), through(&b), A.take(2u16));
}
