// expect: L0612
// rustc E0277: the trait gives no `T: Copy`, so `Need<T>` is not
// well-formed as the const's type.
struct Need<T: Copy>(T);
trait Bad<T> {
    const C: Need<T>;
}
fn main() {}
