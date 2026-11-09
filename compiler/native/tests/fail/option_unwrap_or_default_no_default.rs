// expect: L0612
// rustc E0277: unwrap_or_default needs T: Default.
struct N;
fn main() { let o: Option<N> = None; let _n = o.unwrap_or_default(); }
