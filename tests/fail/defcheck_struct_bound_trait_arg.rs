// expect: L0612
// rustc E0277: a never-built struct's bound `Sup<String>` owes
// `String: Copy`.
trait Sup<T: Copy> {}
struct S<T: Sup<String>>(T);
fn main() {}
