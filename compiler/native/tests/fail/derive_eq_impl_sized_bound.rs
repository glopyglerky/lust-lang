// expect: L0112
// rustc E0369: the written `PartialEq` has an implicit `S: Sized` bound, and the
// field's `S` is `[u8]`, so the derived `PartialEq` cannot compare the field.
#[allow(dead_code)]
struct M<K, S: ?Sized> { k: Vec<K>, s: Box<S> }
impl<K: PartialEq, S> PartialEq for M<K, S> {
    fn eq(&self, o: &Self) -> bool { self.k == o.k }
}
#[derive(PartialEq)]
struct Outer { m: M<u8, [u8]> }
fn main() { let o = Outer { m: M { k: vec![1], s: vec![1u8].into_boxed_slice() } }; println!("{}", o == o); }
