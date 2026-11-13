// expect: L0612
// rustc E0277: `Box<T>: Default` needs `T: Default` (the empty-slice
// impl is only for `Box<[T]>`).
struct NoDef;
fn main() {
    let b: Box<NoDef> = Box::default();
    let _ = b;
}
