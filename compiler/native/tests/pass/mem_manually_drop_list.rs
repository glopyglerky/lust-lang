// ts-rust checker::types SharedList (TRIAGE R2, R21): a ManuallyDrop enum with
// a user Drop that drops one variant through DerefMut, and its size asserts.
use std::mem::ManuallyDrop;
use std::rc::Rc;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct TypeId(u32);
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Flags(u32);

pub struct SharedList<T: 'static> {
    repr: ManuallyDrop<SharedListRepr<T>>,
}

const SHARED_LIST_INLINE: usize = 3;

enum SharedListRepr<T: 'static> {
    Arena(&'static [T]),
    Owned(Rc<[T]>),
    Inline { len: InlineLen, items: [T; SHARED_LIST_INLINE] },
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum InlineLen {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

const _: () = assert!(std::mem::size_of::<SharedList<TypeId>>() == 24);
const _: () = assert!(std::mem::size_of::<SharedList<Flags>>() == 24);
const SZ: [usize; 6] = [
    std::mem::size_of::<SharedListRepr<TypeId>>(),
    std::mem::size_of::<Option<SharedList<TypeId>>>(),
    std::mem::align_of::<SharedList<TypeId>>(),
    std::mem::size_of::<Option<ManuallyDrop<SharedListRepr<u8>>>>(),
    std::mem::size_of::<ManuallyDrop<InlineLen>>(),
    std::mem::size_of::<Option<ManuallyDrop<InlineLen>>>(),
];

impl<T> SharedList<T> {
    fn from_repr(repr: SharedListRepr<T>) -> Self {
        Self { repr: ManuallyDrop::new(repr) }
    }
}

impl<T: Copy + Default> SharedList<T> {
    fn inline(items: &[T]) -> Self {
        let d = T::default();
        let (len, buffer): (InlineLen, [T; SHARED_LIST_INLINE]) = match *items {
            [] => (InlineLen::Zero, [d, d, d]),
            [a] => (InlineLen::One, [a, d, d]),
            [a, b] => (InlineLen::Two, [a, b, d]),
            [a, b, c] => (InlineLen::Three, [a, b, c]),
            _ => unreachable!("inline list longer than SHARED_LIST_INLINE"),
        };
        Self::from_repr(SharedListRepr::Inline { len, items: buffer })
    }
    fn heap(items: &[T]) -> Self {
        Self::from_repr(SharedListRepr::Owned(Rc::from(items)))
    }
    fn arena(items: &'static [T]) -> Self {
        Self::from_repr(SharedListRepr::Arena(items))
    }
    pub fn slice(&self, range: std::ops::Range<usize>) -> Self {
        assert!(range.start <= range.end && range.end <= self.len());
        if range.len() <= SHARED_LIST_INLINE {
            return Self::inline(&self[range]);
        }
        match &*self.repr {
            SharedListRepr::Arena(items) => Self::from_repr(SharedListRepr::Arena(&items[range])),
            SharedListRepr::Owned(items) => Self::from_repr(SharedListRepr::Owned(Rc::from(&items[range]))),
            SharedListRepr::Inline { .. } => unreachable!(),
        }
    }
    fn owners(&self) -> usize {
        match &*self.repr {
            SharedListRepr::Owned(items) => Rc::strong_count(items),
            _ => 0,
        }
    }
}

impl<T: Copy> Clone for SharedList<T> {
    fn clone(&self) -> Self {
        Self::from_repr(match &*self.repr {
            SharedListRepr::Arena(items) => SharedListRepr::Arena(items),
            SharedListRepr::Owned(items) => clone_owned(items),
            SharedListRepr::Inline { len, items } => SharedListRepr::Inline { len: *len, items: *items },
        })
    }
}

fn clone_owned<T: 'static>(items: &Rc<[T]>) -> SharedListRepr<T> {
    SharedListRepr::Owned(items.clone())
}

impl<T: 'static> Drop for SharedList<T> {
    fn drop(&mut self) {
        if matches!(*self.repr, SharedListRepr::Owned(_)) {
            println!("drop owned");
            drop_owned(&mut self.repr);
        }
    }
}

fn drop_owned<T: 'static>(repr: &mut SharedListRepr<T>) {
    drop(std::mem::replace(repr, SharedListRepr::Arena(&[])));
}

impl<T> std::ops::Deref for SharedList<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        match &*self.repr {
            SharedListRepr::Arena(items) => items,
            SharedListRepr::Owned(items) => items,
            SharedListRepr::Inline { len, items } => &items[..*len as usize],
        }
    }
}

static ARENA: [TypeId; 5] = [TypeId(1), TypeId(2), TypeId(3), TypeId(4), TypeId(5)];

fn main() {
    println!("{:?}", SZ);
    let a = SharedList::inline(&[TypeId(7), TypeId(8)]);
    println!("{:?} {}", &a[..], a.len());
    let h = SharedList::heap(&[Flags(1), Flags(2), Flags(3), Flags(4), Flags(5)]);
    let h2 = h.clone();
    println!("{:?} owners {}", &h2[1..], h.owners());
    let s = h.slice(1..5);
    let t = h.slice(0..2);
    println!("{:?} {:?} {} {}", &*s, &*t, h.owners(), s.owners());
    drop(h2);
    println!("owners {}", h.owners());
    drop(s);
    let ar = SharedList::arena(&ARENA);
    let ar2 = ar.slice(0..4);
    println!("{:?} {:?}", &*ar2, ar.clone().first());
    drop(h);
    println!("end {}", t.len());
}
