// Impls with const params select: the probe gives each const param a
// const variable that unifies with the array length, as rustc does
// (`impl<T, const N: usize> Array for [T; N]`, smallvec's own shape).
pub unsafe trait Array {
    type Item;
    fn size() -> usize;
}
unsafe impl<T, const N: usize> Array for [T; N] {
    type Item = T;
    fn size() -> usize {
        N
    }
}
pub struct SmallVec<A: Array> {
    cap: usize,
    data: Vec<A::Item>,
}
impl<A: Array> SmallVec<A> {
    pub fn new() -> Self {
        SmallVec { cap: A::size(), data: Vec::new() }
    }
    pub fn push(&mut self, x: A::Item) {
        self.data.push(x);
    }
    pub fn spilled(&self) -> bool {
        self.data.len() > self.cap
    }
}
trait Len {
    fn len2(&self) -> usize;
}
impl<T, const N: usize> Len for [T; N] {
    fn len2(&self) -> usize {
        N * 2
    }
}
trait Pair {
    fn n(&self) -> usize;
}
impl<const N: usize> Pair for ([u8; N], [u16; N]) {
    fn n(&self) -> usize {
        N
    }
}
struct Buf<const N: usize> {
    d: [u8; N],
}
trait Cap {
    fn cap(&self) -> usize;
}
impl<const N: usize> Cap for Buf<N> {
    fn cap(&self) -> usize {
        N
    }
}
fn twice<L: Len>(l: &L) -> usize {
    l.len2()
}
fn f<const M: usize>(a: [u8; M]) -> usize {
    a.len2() + M
}
fn main() {
    let mut v: SmallVec<[u8; 2]> = SmallVec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    let mut w = SmallVec::<[String; 8]>::new();
    w.push("x".to_string());
    println!("{} {} {} {}", v.spilled(), v.data.len(), w.spilled(), w.data[0]);
    println!("{} {} {}", [1u8, 2, 3].len2(), [0i64; 5].len2(), twice(&[1u8, 2]));
    println!("{} {} {}", ([1u8, 2], [3u16, 4]).n(), f([1, 2, 3]), f([0; 7]));
    let b = Buf { d: [0u8; 6] };
    let c = Buf::<2> { d: [1, 2] };
    println!("{} {} {}", b.cap(), c.cap(), b.d.len() + c.d[1] as usize);
}
