// expect: L0612
// rustc E0277: the alias itself is lazy, but writing it in a
// signature owes the target's bound.
struct Need<T: Copy>(T);
type Bad = Need<String>;
fn f(_x: Bad) {}
fn main() {}
