// expect: L0612
// rustc E0277: a written projection in a body owes the impl's where clauses.
trait Tr { type Out; }
impl<T: Copy> Tr for Vec<T> { type Out = T; }
fn main() { let x: <Vec<String> as Tr>::Out = String::new(); println!("{}", x.len()); }
