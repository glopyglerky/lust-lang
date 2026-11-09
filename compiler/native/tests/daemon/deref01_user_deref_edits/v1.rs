// User Deref edits on one warm daemon: the deref body (its side effect
// runs at each method call, field read, and index), a DerefMut impl
// added for writes through it, the target type changed, and the impl
// removed (an error).
use std::ops::Deref;
struct Late {
    index: usize,
    items: Vec<u32>,
}
struct Info {
    late: Late,
}
impl Deref for Info {
    type Target = Late;
    fn deref(&self) -> &Late {
        &self.late
    }
}
fn main() {
    let info = Info { late: Late { index: 3, items: vec![1, 2, 3] } };
    println!("{} {}", info.index, info.items.len());
}
