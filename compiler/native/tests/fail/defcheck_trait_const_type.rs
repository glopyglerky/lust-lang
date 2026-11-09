// expect: L0612
// rustc E0277: a required trait const's written type must be
// well-formed at the trait definition, with no impl or use.
struct Need<T: Copy>(T);
trait Bad {
    const C: Need<String>;
}
fn main() {}
