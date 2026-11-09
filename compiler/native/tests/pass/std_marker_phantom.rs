// `std::marker::PhantomData` is lust std source (library/core/src/
// marker.rs, W-STD-C), reached through the `core::marker` overlay. It is
// zero-sized, `Copy`/`Clone`/`Default`/`Eq`/`Ord` with no bounds on its
// `T`, a unit value whose `T` comes from the expected type, and usable
// in const fns and statics. Rustc's derive bounds still apply to user
// derives around it.
use std::marker::PhantomData;

#[derive(Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
struct Id<T> {
    raw: u32,
    _t: PhantomData<T>,
}

struct Key<T> {
    id: u32,
    _t: PhantomData<fn() -> T>,
}

impl<T> Key<T> {
    const fn new(id: u32) -> Key<T> {
        Key { id, _t: PhantomData }
    }
}

struct NotSend {
    n: u8,
    _not_send: std::marker::PhantomData<*const ()>,
}

static KEY: Key<String> = Key::new(7);
const PH: PhantomData<i64> = PhantomData;

struct Iter<'a, T> {
    i: usize,
    _m: PhantomData<&'a T>,
}

fn iter_of<'a, T>(_v: &'a [T]) -> Iter<'a, T> {
    Iter { i: 0, _m: PhantomData }
}

fn main() {
    let a: Id<u8> = Id { raw: 1, _t: PhantomData };
    let b = a;
    let c = Id::<u8>::default();
    println!("{} {} {} {}", a == b, a < c, c.raw, a.max(c).raw);
    let s: Id<String> = Id { raw: 2, _t: PhantomData };
    let t = s.clone();
    println!("{}", s == t);
    println!("{} {}", std::mem::size_of::<PhantomData<u64>>(), std::mem::align_of::<PhantomData<u64>>());
    println!("{}", std::mem::size_of::<Id<u64>>());
    let p: PhantomData<String> = PhantomData;
    let q = p;
    let r = p.clone();
    println!("{} {} {}", p == q, q <= r, PH == core::marker::PhantomData::<i64>);
    println!("{} {}", KEY.id, Key::<u8>::new(3).id);
    let ns = NotSend { n: 4, _not_send: PhantomData };
    println!("{}", ns.n);
    let v = vec![1, 2, 3];
    let it = iter_of(&v);
    println!("{} {}", it.i, v.len());
}
