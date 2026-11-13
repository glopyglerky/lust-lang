// A bound proved by a blanket impl through the caller's bound: the instance
// calls the blanket's method, never the inherent method of the concrete type.
trait A { fn f(&self) -> u8; }
trait B { fn b(&self) -> u8; }
impl<X: B> A for X { fn f(&self) -> u8 { self.b() + 10 } }
struct S;
impl S { fn f(&self) -> u8 { 99 } }
impl B for S { fn b(&self) -> u8 { 1 } }
fn h<T: A>(t: &T) -> u8 { t.f() }
fn g<U: B>(u: &U) -> u8 { h(u) }
fn main() { println!("{} {}", g(&S), S.f()); }
