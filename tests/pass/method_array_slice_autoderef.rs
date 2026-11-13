// A trait (and inherent-free) method on `[T]` reached from an
// array receiver by rustc's array-to-slice autoderef step: byte
// string literals, array places, references, mutable receivers,
// a user `Deref` target, and a generic body whose instance
// replays the pick.
trait Probe {
    fn total(&self) -> u32;
    fn bump(&mut self);
}

impl Probe for [u8] {
    fn total(&self) -> u32 {
        let mut s = 0u32;
        let mut i = 0;
        while i < self.len() {
            s += self[i] as u32;
            i += 1;
        }
        s
    }
    fn bump(&mut self) {
        let mut i = 0;
        while i < self.len() {
            self[i] += 1;
            i += 1;
        }
    }
}

trait GenericProbe<T> {
    fn tally(&self) -> usize;
}

impl<T> GenericProbe<T> for [T] {
    fn tally(&self) -> usize {
        self.len()
    }
}

fn via_generic<T>(t: T) -> T {
    // A concrete array receiver inside a generic body: the
    // template records the slice pick and instances replay it.
    println!("{}", b"ab\0".total());
    t
}

fn main() {
    println!("{}", b"ab\0".total());
    let a = [1u8, 2, 3];
    println!("{}", a.total());
    println!("{}", (&a).total());
    let r = &a;
    println!("{}", r.total());
    let rr = &r;
    println!("{}", rr.total());
    let mut b = [10u8, 20];
    b.bump();
    println!("{}", b.total());
    (&mut b).bump();
    println!("{}", b.total());
    let boxed = Box::new([5u8, 6, 7, 8]);
    println!("{}", boxed.total());
    println!("{}", [9u16, 8, 7].tally());
    println!("{}", via_generic(41) + 1);
    // UFCS naming the slice impl coerces the array reference.
    println!("{}", <[u8] as Probe>::total(&a));
    // A slice receiver still uses the plain autoref path.
    let s: &[u8] = &a[1..];
    println!("{}", s.total());
}
