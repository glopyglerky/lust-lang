// expect: L0601
// rustc E0391: the defaults of A and B name each other (a cycle).
struct A<T = B>(Option<Box<T>>);
struct B<U = A>(Option<Box<U>>);
fn main() {}
