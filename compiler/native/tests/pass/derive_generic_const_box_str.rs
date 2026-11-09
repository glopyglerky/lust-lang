// Derived Clone of a const-generic Adt over an enum that holds a
// `Box<str>` (jiff's SmallStr), and Clone through `Rc`/`Arc` of a type with
// no Clone, `Box<str>`, and `Box<[T]>`: std's impls ask nothing of `str`
// or of the `Rc` pointee, so the derived impls hold at the definition.
use std::rc::Rc;
use std::sync::Arc;
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct SmallStr<const M: usize> {
    kind: SmallStrKind<M>,
}
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
enum SmallStrKind<const M: usize> {
    Array(ArrayStr<M>),
    Static(&'static str),
    Heap(Box<str>),
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ArrayStr<const N: usize> {
    len: u8,
    bytes: [u8; N],
}
struct NoClone(u8);
#[derive(Clone)]
struct Shared<T> {
    r: Rc<NoClone>,
    a: Arc<NoClone>,
    s: Box<str>,
    b: Box<[T]>,
}
fn main() {
    let a: SmallStr<4> = SmallStr { kind: SmallStrKind::Heap("heap".into()) };
    let b = SmallStr::<4> { kind: SmallStrKind::Array(ArrayStr { len: 1, bytes: [7; 4] }) };
    let c = SmallStr::<4> { kind: SmallStrKind::Static("st") };
    println!("{:?} {:?} {:?}", a.clone(), b.clone(), c);
    println!("{} {:?}", a == a.clone(), b.cmp(&c));
    let s = Shared { r: Rc::new(NoClone(1)), a: Arc::new(NoClone(2)), s: "x".into(), b: vec![3u8, 4].into_boxed_slice() };
    let t = s.clone();
    println!("{} {} {} {:?}", t.r.0, t.a.0, t.s, t.b);
}
