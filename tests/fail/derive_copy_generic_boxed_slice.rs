// expect: L0112
// rustc E0204: `Box<[T]>` is never Copy, so `impl<T: Copy> Copy for S<T>`
// cannot hold, used or not.
#[derive(Clone, Copy)]
struct S<T> {
    b: Box<[T]>,
}
fn main() {
    let s: S<u8> = S { b: vec![1].into_boxed_slice() };
    println!("{}", s.b.len());
}
