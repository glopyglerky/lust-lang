// expect: L0612
struct W<T> { v: Vec<T> }
unsafe impl<T: Copy + Send> Send for W<T> {}
fn need<T: Send>() {}
fn main() { need::<W<String>>(); }
