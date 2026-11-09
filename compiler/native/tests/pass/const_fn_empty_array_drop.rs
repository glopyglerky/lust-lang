// An empty array has no destructor, whatever its element type, so a const
// fn may make and drop one (`&[]` of a generic element, a `[T; 0]` local),
// as in ts-rust checker/types.rs `SharedList::empty_non_nil`.
use std::mem::ManuallyDrop;
use std::rc::Rc;
enum Repr<T: 'static> {
    Arena(&'static [T]),
    Owned(Rc<[T]>),
}
struct L<T: 'static> {
    repr: ManuallyDrop<Repr<T>>,
}
impl<T> L<T> {
    const fn empty() -> Self {
        Self { repr: ManuallyDrop::new(Repr::Arena(&[])) }
    }
    fn len(&self) -> usize {
        match &*self.repr {
            Repr::Arena(a) => a.len(),
            Repr::Owned(r) => r.len(),
        }
    }
}
const fn e0<T>() -> [T; 0] {
    []
}
const fn e1<T>() -> usize {
    let a: [T; 0] = [];
    let _ = &a;
    0
}
const C: usize = {
    let a: [String; 0] = [];
    a.len() + 1
};
fn main() {
    let l: L<String> = L::empty();
    println!("{}", l.len());
    let a: [String; 0] = e0();
    let b: [Vec<u8>; 0] = [];
    println!("{} {} {} {}", a.len(), b.len(), e1::<String>(), C);
}
