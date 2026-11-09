// alloc's `impl<T> Default for Box<[T]>` (the empty boxed slice, no
// bound on `T`) and `FromIterator<T> for Box<[T]>`: derive(Default)
// over the field, every `default()` spelling, collect and from_iter,
// and drops of the empty value.
#[derive(Default, Debug)]
struct Table {
    index: Box<[u16]>,
    slots: Box<[(u8, String)]>,
}
#[derive(Default, Debug)]
struct Wrap<T> {
    b: Box<[T]>,
    o: Option<Box<[T]>>,
}
struct Noisy(u8);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn make<T: Default>() -> T {
    T::default()
}
fn main() {
    println!("{:?}", Table::default());
    println!("{:?}", Wrap::<u8>::default());
    let a: Box<[u8]> = Box::<[u8]>::default();
    let b: Box<[String]> = Box::default();
    let c: Box<[u32]> = Default::default();
    let d: Box<[i64]> = make();
    println!("{:?} {:?} {:?} {:?} {}", a, b, c, d, a.is_empty());
    let e: Box<[u8]> = [1u8, 2, 3].into_iter().collect();
    let f: Box<[String]> = Box::from_iter(["x", "y"].iter().map(|s| s.to_string()));
    let g: Box<[i32]> = (0..4).collect::<Box<[i32]>>();
    println!("{:?} {:?} {:?} {:?}", e, f, g, &g[1..3]);
    let v = e.into_vec();
    println!("{} {}", v.len(), v.capacity());
    let mut h: Box<[Noisy]> = vec![Noisy(1), Noisy(2)].into_boxed_slice();
    let old = std::mem::take(&mut h);
    println!("{} {}", old.len(), h.len());
    drop(old);
    println!("done");
}
