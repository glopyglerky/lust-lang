// Body edits that add or drop generic Adt types (slice::Iter<u64> by `.iter().sum()`, Bag<u64>,
// Holder<u8> with a printing Drop): the kept program takes them when every instance they need was
// made before the body's discovery turn (by an earlier body or a field), and a full collect when
// a later body or the edited body itself makes one (v12, v13, v15, v19-v21).
mod m { pub const R: u32 = 1; }
#[derive(Debug, Clone)]
struct Bag<T> { items: Vec<T> }
struct Holder<T: std::fmt::Debug>(Option<T>);
impl<T: std::fmt::Debug> Drop for Holder<T> {
    fn drop(&mut self) { println!("drop {:?}", self.0); }
}
fn early() -> u64 {
    let a: Vec<u64> = vec![1, 2];
    let b: Vec<u32> = vec![3];
    let s: u64 = a.iter().sum();
    let t: u32 = b.iter().sum();
    let g: Bag<u64> = Bag { items: vec![5] };
    let h = Holder(Some(1u8));
    s + t as u64 + g.items.len() as u64 + m::R as u64
}
fn helper(x: u32) -> u64 {
    let mut acc = x as u64;
    // slot
    let g: Bag<u64> = Bag { items: vec![x as u64] }; acc += g.items[0];
    acc
}
fn late() -> u64 {
    let k: Vec<u16> = vec![4];
    let s: u16 = k.iter().sum();
    let q: Bag<i8> = Bag { items: vec![1] };
    s as u64 + q.items.len() as u64
}
fn main() {
    println!("{} {} {}", early(), helper(4), late());
}
