// expect: L0612
// rustc E0277: a field's projection owes the impl's where clauses.
trait Tr { type Out; }
impl<T: Copy> Tr for Vec<T> { type Out = T; }
struct S { x: <Vec<String> as Tr>::Out }
fn main() { let s = S { x: String::new() }; println!("{}", s.x.len()); }
