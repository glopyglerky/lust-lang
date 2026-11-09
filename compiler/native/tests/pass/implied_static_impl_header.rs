// An impl header type implies its Adt's `T: 'static` bound (rustc implied
// bounds), so building the Adt in the impl needs no written bound.
struct S<T: 'static> { r: R<T> }
enum R<T: 'static> { A(&'static [T]), B(u8) }
impl<T> S<T> {
    fn mk(r: R<T>) -> Self { S { r } }
}
impl<T: Copy> S<T> {
    fn b() -> Self { Self::mk(R::B(1)) }
}
fn main() { let s: S<u32> = S::b(); if let R::B(x) = s.r { println!("{x}"); } let _ = R::<u8>::A(&[]); }
