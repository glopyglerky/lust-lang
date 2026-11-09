// expect: L0612
// rustc E0277: `N` does not implement `Debug`.
struct N;
fn main() {
    let d: &dyn std::fmt::Debug = &N;
    let _ = d;
}
