// expect: L0612
// rustc E0277: the Adt's bound holds at the constructor value.
#[derive(Clone, Copy)]
struct P<T: Copy>(T, T);
fn main() {
    let f = P::<String>;
    let _ = f;
}
