// Generic const fns with trait and lifetime bounds in statics and
// consts (ts-rust `ContextKey::new`): the bounds are proved at the
// concrete type args.
use std::marker::PhantomData;
pub struct Key<T> {
    name: &'static str,
    _v: PhantomData<fn() -> T>,
}
impl<T: 'static> Key<T> {
    pub const fn new(name: &'static str) -> Key<T> {
        Key { name, _v: PhantomData }
    }
}
struct K<T>(u32, PhantomData<T>);
impl<T: Clone + Send + 'static> K<T> {
    const fn new(v: u32) -> K<T> {
        K(v, PhantomData)
    }
}
trait Tr {}
#[derive(Clone)]
struct Loc(String);
impl Tr for Loc {}
struct M<T>(u32, PhantomData<T>);
impl<T: Tr> M<T> {
    const fn new(v: u32) -> M<T> {
        M(v, PhantomData)
    }
}
static KEY: Key<String> = Key::new("contextKey(0)");
static A: K<String> = K::new(1);
static B: K<Loc> = K::new(2);
static C: K<(u8, Vec<u8>)> = K::new(3);
static D: K<[u8; 2]> = K::new(4);
const E: u32 = {
    let m: M<Loc> = M::new(5);
    m.0
};
fn main() {
    println!("{} {} {} {} {} {}", KEY.name, A.0, B.0, C.0, D.0, E);
}
