// Projections in the signatures and fields of a generic Adt
// (the smallvec adapter shape): `A::Item` params, returns, struct
// literal fields, field reads, patterns, and variant payloads are
// normalized where the Adt's args are known (rustc normalizes at
// each use).
pub unsafe trait Array {
    type Item;
    fn size() -> usize;
}
macro_rules! impl_array {
    ($($n:expr),*) => { $(unsafe impl<T> Array for [T; $n] { type Item = T; fn size() -> usize { $n } })* }
}
impl_array!(1, 2, 4, 8);

pub struct SmallVec<A: Array> {
    cap: usize,
    data: Vec<A::Item>,
}

impl<A: Array> SmallVec<A> {
    pub fn new() -> SmallVec<A> {
        SmallVec { cap: A::size(), data: Vec::new() }
    }
    pub fn push(&mut self, value: A::Item) {
        self.data.push(value);
    }
    pub fn pop(&mut self) -> Option<A::Item> {
        self.data.pop()
    }
    pub fn first(&self) -> &A::Item {
        &self.data[0]
    }
    pub fn as_slice(&self) -> &[A::Item] {
        &self.data
    }
    pub fn spilled(&self) -> bool {
        self.data.len() > self.cap
    }
    pub fn with(x: A::Item, y: A::Item) -> Self {
        SmallVec { cap: A::size(), data: vec![x, y] }
    }
}

impl<A: Array> SmallVec<A>
where
    A::Item: Copy,
{
    pub fn from_slice(s: &[A::Item]) -> Self {
        let mut v = SmallVec::new();
        for &x in s {
            v.push(x);
        }
        v
    }
}

struct Pair<A: Array> {
    v: Vec<A::Item>,
    n: A::Item,
}

struct One<A: Array>(A::Item, usize);

enum E<A: Array> {
    One(A::Item),
    Two { a: A::Item, b: A::Item },
    Zero,
}

fn total(p: &Pair<[u32; 4]>) -> u32 {
    p.v.iter().sum::<u32>() + p.n
}

fn main() {
    let mut v: SmallVec<[u16; 4]> = SmallVec::new();
    v.push(7);
    v.push(40000);
    for i in 0..4 {
        v.push(i * 3);
    }
    let f = *v.first();
    println!("{} {} {}", f + 1, v.spilled(), v.as_slice().len());
    let x = v.pop().unwrap();
    println!("{} {}", x.count_ones(), f.leading_zeros());

    let w = SmallVec::<[u8; 2]>::from_slice(&[1, 2, 3]);
    let q: SmallVec<[u32; 8]> = SmallVec::with(70000, 1);
    println!("{} {} {}", w.spilled(), w.as_slice()[2].wrapping_add(255), q.as_slice()[0] + q.as_slice()[1]);

    let p: Pair<[u32; 4]> = Pair { v: vec![1, 2, 3], n: 10 };
    let k = p.n;
    let r = Pair::<[i8; 4]> { v: Vec::new(), n: -1 };
    println!("{} {} {} {}", total(&p), k.count_ones(), r.n, r.v.len());

    let o: One<[u8; 4]> = One(250, 1);
    let t = One::<[i64; 4]>(-9, 2);
    println!("{} {}", o.0.wrapping_add(10) as usize + o.1, t.0 * t.1 as i64);

    let e: E<[u8; 4]> = E::One(5);
    let g = E::<[i32; 4]>::Two { a: -1, b: 2 };
    let z: E<[u8; 4]> = E::Zero;
    if let E::One(x) = e {
        println!("{}", x.wrapping_add(251));
    }
    if let E::Two { a, b } = g {
        println!("{}", a * b);
    }
    if let E::Zero = z {
        println!("zero");
    }
}
