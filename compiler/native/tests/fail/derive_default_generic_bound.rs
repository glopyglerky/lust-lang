// expect: L0612
// rustc E0277: derived Default bounds T: Default.
#[derive(Default)]
struct S<T>(Vec<T>);
struct N;
fn main() { let s: S<N> = S::default(); println!("{}", s.0.len()); }
