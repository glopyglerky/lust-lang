// expect: L0612
// rustc E0277: a never-called fn's bound `Sup<String>` owes
// `String: Copy`.
trait Sup<T: Copy> {}
fn f<T: Sup<String>>(_x: T) {}
fn main() {}
