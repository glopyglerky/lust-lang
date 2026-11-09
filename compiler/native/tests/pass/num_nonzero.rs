// `core::num::NonZero<T>` and its aliases (TRIAGE T5): new and
// get, Debug and Display forward to the integer, derived
// comparisons, and the zero niche in `Option<NonZero>` layout.
use std::num::{NonZero, NonZeroI32, NonZeroU32, NonZeroU64, NonZeroUsize};

fn read<T>(v: T) -> T {
    v
}

struct Id(NonZeroU64);

fn main() {
    let a = NonZeroU32::new(read(5)).unwrap();
    println!("{} {:?} {:?}", a.get(), NonZeroU64::new(read(0)), NonZeroUsize::new(read(3)));
    let b = NonZeroI32::new(read(-2)).unwrap();
    println!("{} {:?} {}", b.get(), b, a);
    println!("{}", a == NonZeroU32::new(5).unwrap() && a < NonZeroU32::new(6).unwrap());
    let c: Option<NonZero<u64>> = NonZero::new(read(9u64));
    println!("{:?} {:?}", c, c.map(NonZero::get));
    let v = vec![a, NonZeroU32::new(8).unwrap()];
    println!("{:?} {}", v, v.iter().map(|n| n.get()).sum::<u32>());
    let id = Id(NonZeroU64::new(read(41)).unwrap());
    println!("{}", id.0.get() + 1);
    println!("{} {}", std::mem::size_of::<Option<NonZeroU32>>(), std::mem::size_of::<Option<Id>>());
    let none: Option<NonZeroU32> = NonZeroU32::new(0);
    println!("{:?} {}", none, none.is_none());
    println!("{:?} {:?} {:?} {:?}", NonZeroU32::MIN, NonZeroU32::MAX, NonZeroI32::MIN, NonZeroI32::MAX);
    println!("{:?}", NonZero::<u64>::MAX.get());
    const M: NonZeroU32 = NonZeroU32::MIN;
    println!("{}", M.get());
    println!("{}", NonZeroU32::new(1).unwrap() == NonZeroU32::MIN);
}
