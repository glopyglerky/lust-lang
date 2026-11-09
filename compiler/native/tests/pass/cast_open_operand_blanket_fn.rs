// A cast whose operand type is fixed only after the closure body: the
// closure parameter's type comes from the blanket impl's `FnMut(u32)` bound
// when the call's `impl Sink` obligation is selected (rustc checks casts at
// body end). ts-rust ast/store.rs `set_parent_in_owned_store_children`.
pub trait StoreChildSink {
    fn child(&mut self, index: u32) -> bool;
}
impl<F: FnMut(u32) -> bool> StoreChildSink for F {
    fn child(&mut self, index: u32) -> bool {
        self(index)
    }
}
fn for_each_store_child_id(ids: &[u32], mut v: impl StoreChildSink) -> bool {
    for &i in ids {
        if v.child(i) {
            return true;
        }
    }
    false
}
fn main() {
    let nodes: Vec<Option<u8>> = vec![Some(1), None, Some(3), Some(4)];
    let mut seen = Vec::new();
    let stopped = for_each_store_child_id(&[0, 2, 3, 1, 2], |child| {
        let child = child as usize;
        if nodes[child].is_none() {
            return true;
        }
        seen.push(child);
        false
    });
    let d = Default::default();
    let x = d as f64;
    let _: u8 = d;
    println!("{} {:?} {}", stopped, seen, x);
}
