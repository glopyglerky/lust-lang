// expect: L0001
// rustc rejects an unbalanced delimiter in a statement attribute inside a body.
fn main() {
    #[cfg_attr(any(), allow(unused)]
    let x = 1;
    let _ = x;
}
