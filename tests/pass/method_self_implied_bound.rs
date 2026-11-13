// A method's `&'a self` gives `Self: 'a`, so `T: 'a` for `impl<T> C<T>`: a bound `impl Fn(&'a T, usize)
// -> &'a E` needs nothing more (ts-rust ast/synthetic.rs ChunkTable::far_entry).
use std::collections::HashMap;
const FREED: &str = "freed";
struct Chunks<T> { base: usize, near: Vec<Option<T>>, far: HashMap<usize, T> }
impl<T> Chunks<T> {
    fn entry<'a, const PER: usize, E: ?Sized>(&'a self, index: usize, pick: impl Fn(&'a T, usize) -> &'a E) -> &'a E {
        match self.near.get((index / PER).wrapping_sub(self.base)) {
            Some(Some(chunk)) => pick(chunk, index % PER),
            _ => self.far_entry::<PER, E>(index, pick),
        }
    }
    fn far_entry<'a, const PER: usize, E: ?Sized>(&'a self, index: usize, pick: impl Fn(&'a T, usize) -> &'a E) -> &'a E {
        pick(self.far.get(&(index / PER)).expect(FREED), index % PER)
    }
}
fn main() {
    let mut c: Chunks<[u32; 4]> = Chunks { base: 0, near: vec![Some([1, 2, 3, 4])], far: HashMap::new() };
    c.far.insert(5, [9, 8, 7, 6]);
    println!("{} {}", c.entry::<4, u32>(2, |ch, i| &ch[i]), c.entry::<4, u32>(21, |ch, i| &ch[i]));
}
