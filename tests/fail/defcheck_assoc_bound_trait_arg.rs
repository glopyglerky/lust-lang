// expect: L0612
// rustc E0277: an associated type bound `Sup<String>` owes
// `String: Copy` at the trait definition.
trait Sup<T: Copy> {}
trait Bad {
    type A: Sup<String>;
}
fn main() {}
