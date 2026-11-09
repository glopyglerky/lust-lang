// expect: L0102
trait A { type Out; }
trait B { type Out; }
trait C: A + B {}
fn f<T: C<Out = i32>>(x: T) -> i32 { 1 }
fn main() { println!("x"); }
