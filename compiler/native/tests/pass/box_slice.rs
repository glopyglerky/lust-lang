// Box<[T]> from Vec::into_boxed_slice: Debug, Clone, Eq, indexing and
// slice methods through deref, into_vec, drops, and its 16-byte size.
fn total(s: &[i32]) -> i32 {
    s.iter().sum()
}
struct Noisy(u8);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let b: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();
    let d: Box<[String]> = vec![String::from("a"), String::from("b")].into_boxed_slice();
    let mut f = b.clone();
    f[0] = 10;
    println!("{:?} {:?} {:?}", b, d, f);
    println!("{} {} {} {} {}", b.len(), total(&b), b[2], b == f, b.is_empty());
    for x in d.iter() {
        print!("{} ", x);
    }
    println!();
    let firsts: Vec<i32> = f.iter().map(|x| x + 1).collect();
    println!("{:?} {:?} {:?}", firsts, &f[1..], f.first());
    let v = d.into_vec();
    println!("{:?} {} {}", v, v.len(), v.capacity());
    let e: Box<[u8]> = Vec::new().into_boxed_slice();
    println!("{:?} {}", e, e.len());
    let n: Box<[Noisy]> = vec![Noisy(1), Noisy(2)].into_boxed_slice();
    println!("{}", n.len());
    drop(n);
    let mut w = Vec::with_capacity(10);
    w.push(5u64);
    let wb = w.into_boxed_slice();
    let back = wb.into_vec();
    println!("{} {} {}", back.len(), back.capacity(), std::mem::size_of::<Box<[u64]>>());
}
