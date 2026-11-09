// Zero-sized values that hold a zero-length array, returned from fns,
// trait methods, closures, fn pointers, and derived Default/Clone glue.
#[derive(Default, Clone, Debug)]
struct W {
    a: [u8; 0],
    b: ((), [u32; 0]),
}
#[derive(Clone, Debug, Default)]
struct M;
#[derive(Clone, Debug, Default)]
struct V {
    z: [M; 0],
    w: W,
}
trait Mk {
    fn mk(&self) -> [u64; 0];
}
impl Mk for u8 {
    fn mk(&self) -> [u64; 0] {
        []
    }
}
fn make<T: Default>() -> T {
    T::default()
}
fn take(f: fn() -> [i8; 0]) -> [i8; 0] {
    f()
}
fn e() -> [i8; 0] {
    []
}
fn twice(w: &W) -> (W, W) {
    (w.clone(), w.clone())
}
fn main() {
    let w = W::default();
    let v = V::default();
    let c = v.clone();
    let k: Box<dyn Mk> = Box::new(3u8);
    let cl = || -> [u16; 0] { Default::default() };
    let g: [u32; 0] = make();
    let t = twice(&w);
    println!("{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}", w, c, k.mk(), cl(), g, take(e), t, 3u8.mk());
    let arr: [[u8; 0]; 3] = Default::default();
    println!("{:?} {:?}", arr, arr.clone());
}
