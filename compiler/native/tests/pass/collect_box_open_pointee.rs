// collect into Box<T> whose T only a later coercion fixes (ts-rust ast/store.rs
// adopt_detached_store: Box::leak(it.collect()) as &'static [Node]) waits for body end.
use std::rc::Rc;
// The leaked slices stay reachable from a static, so the sanitized build sees no leak.
static KEEP: std::sync::Mutex<Vec<&'static [N]>> = std::sync::Mutex::new(Vec::new());
#[derive(Clone, Copy, Debug, PartialEq)] struct N(u32);
fn remap(n: N) -> N { N(n.0 + 100) }
fn main() {
    let src: &'static [N] = Box::leak(vec![N(1), N(2)].into_boxed_slice());
    let jsdocs: &'static [N] = if src.iter().any(|&n| remap(n) != n) { Box::leak(src.iter().map(|&n| remap(n)).collect()) } else { src };
    println!("{:?}", jsdocs);
    let b = Box::leak(src.iter().copied().collect());
    let c: &'static [N] = b;
    println!("{:?}", c);
    KEEP.lock().unwrap().extend([src, jsdocs, c]);
    let r = Rc::new(1); let _ = r;
}
