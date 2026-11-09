// Forms a const initializer's body check must type, now that its errors
// always count (Sol review 3, L1): `<u64>::MAX`, a trait const on a
// primitive, `abs_diff`, `midpoint` as a method, `BuildHasherDefault::new`,
// and a lazy static whose value type holds a `dyn`. The same forms in fn
// bodies.
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, DefaultHasher};
use std::sync::LazyLock;

trait Tr {
    const C: usize;
}
impl Tr for u8 {
    const C: usize = 4;
}

type H = Box<dyn Fn(u32) -> u32 + Send + Sync>;
static LAZY: LazyLock<HashMap<u32, H>> = LazyLock::new(|| {
    let mut m: HashMap<u32, H> = HashMap::new();
    m.insert(1, Box::new(|x| x + 1));
    m
});
static ONE: LazyLock<Box<dyn Fn(u32) -> u32 + Send + Sync>> = LazyLock::new(|| Box::new(|x| x * 3));
const B: u64 = <u64>::MAX - 1;
const TC: usize = u8::C * 2;
const DIFF: u32 = 3u32.abs_diff(10);
const MID: i32 = (-7i32).midpoint(4);
type SipMap = HashMap<u32, u32, BuildHasherDefault<DefaultHasher>>;
static HASHER: BuildHasherDefault<DefaultHasher> = BuildHasherDefault::new();

fn main() {
    println!("{} {}", (LAZY.get(&1).unwrap())(41), (*ONE)(5));
    println!("{} {} {} {}", B, TC, DIFF, MID);
    let local: LazyLock<Box<dyn Fn(u32) -> u32 + Send + Sync>> = LazyLock::new(|| Box::new(|x| x + 100));
    println!("{}", (*local)(1));
    let m: SipMap = HashMap::with_hasher(BuildHasherDefault::new());
    let _h = &HASHER;
    println!("{}", m.len());
    println!("{} {}", <u64>::MAX, <u8>::MIN);
    println!("{} {}", u8::C, <u8 as Tr>::C);
    println!("{} {} {}", 250u8.abs_diff(3), (-100i8).abs_diff(100), i64::MIN.abs_diff(i64::MAX));
    println!("{} {} {}", u32::MAX.midpoint(1), (-9i64).midpoint(2), 7u8.midpoint(200));
}
