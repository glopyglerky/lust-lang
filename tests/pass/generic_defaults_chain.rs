// A default may name another type whose own default applies.
struct A<T = B>(T);
struct B<U = u16>(U);
fn main() { let a: A = A(B(7)); let x: u16 = (a.0).0; println!("{}", x); }
