// User Deref edits on one warm daemon: the deref body (its side effect
// runs at each method call, field read, and index), a DerefMut impl
// added for writes through it, the target type changed, and the impl
// removed (an error).
use std::ops::{Deref, DerefMut};
struct Late {
    pad: u64,
    index: usize,
    items: Vec<u32>,
}
struct Info {
    late: Late,
}
impl DerefMut for Info {
    fn deref_mut(&mut self) -> &mut Late {
        println!("deref_mut");
        &mut self.late
    }
}
fn main() {
    let mut info = Info { late: Late { pad: 0, index: 3, items: vec![1, 2, 3] } };
    info.index = 7;
    info.items.push(4);
    println!("{} {}", info.index, info.items.len());
}
