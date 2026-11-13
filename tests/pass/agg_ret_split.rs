// Returns of 9-16 bytes whose pieces cross byte 8 (release C passes them through the out
// pointer), next to the sizes around them: 8 and 9 bytes, 16 bytes that split cleanly, and
// 17 and 20 bytes. Each goes through direct calls, fn pointers, dyn methods, closures (generic
// and dyn), a generic fn and recursion. tests/unwind/agg_ret_split_unwind.rs has the panics.

type Opt8 = Option<u32>; // 8 bytes: one register
type R9 = Result<[u8; 8], ()>; // 9: the array is bytes 1-8
type Opt12 = Option<(u32, u32)>; // 12: the pair is bytes 4-11
type S16 = (u32, (u32, u32), u32); // 16: rustc puts the pair first, nothing crosses byte 8
type N16 = (u64, u64); // 16: two words
type A16 = [u32; 4]; // 16: four u32, none across byte 8
type S17 = ([u8; 9], [u8; 8]); // 17: the C ABI's memory class
type Opt20 = Option<(u32, u32, u32, u32)>; // 20

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct C16 {
    a: u32,
    pair: (u32, u32), // bytes 4-11
    b: u32,
}

#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, PartialEq)]
struct Al8 {
    tag: u32,
    payload: [u8; 8], // bytes 4-11 of an 8-aligned 16
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tagged {
    Pair((u32, u32)), // bytes 4-11
    One(u8),
    Nothing,
}

#[inline(never)]
fn opt8(x: u32) -> Opt8 {
    if x % 3 == 0 { None } else { Some(x * 7) }
}

#[inline(never)]
fn r9(x: u8) -> R9 {
    if x == 0 { Err(()) } else { Ok([x, x + 1, x + 2, x + 3, x + 4, x + 5, x + 6, x + 7]) }
}

#[inline(never)]
fn opt12(x: u32) -> Opt12 {
    if x % 4 == 0 { None } else { Some((x, x * 2)) }
}

#[inline(never)]
fn s16(x: u32) -> S16 {
    (x, (x + 1, x + 2), x + 3)
}

#[inline(never)]
fn c16(x: u32) -> C16 {
    C16 { a: x, pair: (x + 1, x + 2), b: x + 3 }
}

#[inline(never)]
fn n16(x: u64) -> N16 {
    (x, x << 32)
}

#[inline(never)]
fn a16(x: u32) -> A16 {
    [x, x + 10, x + 20, x + 30]
}

#[inline(never)]
fn s17(x: u8) -> S17 {
    ([x; 9], [x + 1; 8])
}

#[inline(never)]
fn opt20(x: u32) -> Opt20 {
    if x == 0 { None } else { Some((x, x + 1, x + 2, x + 3)) }
}

#[inline(never)]
fn al8(x: u32) -> Al8 {
    Al8 { tag: x, payload: [x as u8; 8] }
}

#[inline(never)]
fn tagged(x: u32) -> Tagged {
    match x % 3 {
        0 => Tagged::Pair((x, x + 100)),
        1 => Tagged::One(x as u8),
        _ => Tagged::Nothing,
    }
}

fn countdown(x: u32) -> Opt12 {
    if x == 0 { Some((0, 0)) } else { countdown(x - 1).map(|(a, b)| (a + x, b + 2 * x)) }
}

fn twice<T: Copy>(x: T) -> Option<(T, T)> {
    Some((x, x))
}

trait Make {
    fn make(&self, x: u32) -> Opt12;
    fn tag(&self, x: u32) -> Tagged {
        tagged(x + 1)
    }
}

struct Maker(u32);

impl Make for Maker {
    fn make(&self, x: u32) -> Opt12 {
        opt12(x + self.0)
    }
}

fn via_generic(f: impl Fn(u32) -> Opt12, x: u32) -> Opt12 {
    f(x)
}

fn via_dyn(f: &dyn Fn(u32) -> S16, x: u32) -> S16 {
    f(x)
}

fn main() {
    for x in 0..5 {
        println!("{x}: {:?} {:?} {:?} {:?}", opt8(x), r9(x as u8), opt12(x), s16(x));
        println!("   {:?} {:?} {:?} {:?}", n16(x as u64), a16(x), s17(x as u8).1, opt20(x));
        println!("   {:?} {:?} {:?}", al8(x), tagged(x), c16(x));
    }
    // Fn pointers.
    let f12: fn(u32) -> Opt12 = opt12;
    let f9: fn(u8) -> R9 = r9;
    let fs: fn(u32) -> S16 = s16;
    let fa: fn(u32) -> Al8 = al8;
    let fc: fn(u32) -> C16 = c16;
    let ptrs: Vec<fn(u32) -> Opt12> = vec![opt12, countdown, |x| Some((x, x))];
    println!("fn ptr {:?} {:?} {:?} {:?} {:?}", f12(7), f9(3), fs(9), fa(2), fc(4));
    println!("fn ptrs {:?}", ptrs.iter().map(|f| f(6)).collect::<Vec<_>>());
    // Dyn methods, a default method, closures.
    let makers: Vec<Box<dyn Make>> = vec![Box::new(Maker(1)), Box::new(Maker(3))];
    println!("dyn {:?} {:?}", makers.iter().map(|m| m.make(4)).collect::<Vec<_>>(), makers[1].tag(4));
    let k = 5;
    println!("closures {:?} {:?}", via_generic(|x| Some((x + k, x)), 2), via_dyn(&|x| (x, (k, k), x), 8));
    let boxed: Box<dyn Fn(u32) -> Opt12> = Box::new(move |x| opt12(x + k));
    println!("boxed {:?} {:?}", boxed(1), boxed(3));
    // Generic and recursive.
    println!("generic {:?} {:?} {:?}", twice(7u32), twice(7u64), twice(1u8));
    println!("recursive {:?}", countdown(6));
}
