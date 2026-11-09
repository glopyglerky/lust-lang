// expect: L0612
// rustc E0277: writing `Sup<String>` owes Sup's parameter bound
// `String: Copy` at the definition.
trait Sup<T: Copy> {}
trait Bad: Sup<String> {}
fn main() {}
