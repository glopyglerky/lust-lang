// expect: L0112
// rustc E0277: the derived Clone of CowChunks<T> clones a Vec<Chunk<T>>,
// and Chunk derives Clone but not Debug, so the derived Debug fails; the
// derived Clone of Pair<T> needs `Chunk2<T>: Clone`, which no impl gives.
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct CowChunks<T: 'static> {
    chunks: Vec<Chunk<T>>,
    tail: Vec<T>,
    len: usize,
}
#[derive(Clone)]
enum Chunk<T: 'static> {
    Fixed(Box<[T; 4]>),
    Shared(Arc<Vec<T>>),
    Freed,
}
#[derive(Clone)]
enum Pair<T> {
    Two(Chunk2<T>, u8),
    None,
}
struct Chunk2<T>(Option<T>);
fn main() {
    let c: CowChunks<u8> = CowChunks { chunks: vec![Chunk::Freed], tail: vec![], len: 0 };
    let p: Pair<u8> = Pair::None;
    let _ = c.clone();
    println!("{} {}", c.len, matches!(p, Pair::None));
}
