// A user `Iterator::size_hint` (W-STD-B): what `collect`, `extend`,
// and the adapters ask of it, and how often, decide the capacities.
struct Noisy(u32);
impl Iterator for Noisy {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 < 6 { self.0 += 1; Some(self.0) } else { None }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        println!("hint at {}", self.0);
        (1, None)
    }
}
struct Plain(u8);
impl Iterator for Plain {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.0 < 5 { self.0 += 1; Some(self.0) } else { None }
    }
}
fn hint_of<I: Iterator>(i: I) -> (usize, Option<usize>) {
    i.size_hint()
}
fn main() {
    let v: Vec<u32> = Noisy(0).collect();
    println!("{:?} {}", v, v.capacity());
    let mut w = vec![0u32];
    w.extend(Noisy(2));
    println!("{:?} {}", w, w.capacity());
    let f: Vec<u32> = Noisy(0).filter(|x| x % 2 == 0).collect();
    println!("{:?} {}", f, f.capacity());
    println!("{:?} {:?}", Plain(0).size_hint(), hint_of(Plain(1)));
    println!("{:?}", hint_of(Noisy(4)));
    let p: Vec<u8> = Plain(0).collect();
    println!("{:?} {}", p, p.capacity());
    let n = Noisy(3);
    println!("{:?}", Iterator::size_hint(&n));
    let s: String = Noisy(0).map(|d| char::from(b'a' + d as u8)).collect();
    println!("{s}");
    let c = Noisy(0).count();
    println!("{c}");
}
