// Box::default() with an inferred T (ts-rust ast/store.rs:3316 Box::leak(Box::default())).
// The leaked value stays reachable from a static, so the sanitized build sees no leak.
use std::sync::Mutex;
#[derive(Default, Debug)]
struct S { a: u8, v: Vec<u8> }
static KEEP: Mutex<Vec<&'static S>> = Mutex::new(Vec::new());
fn get(o: Option<&'static mut S>) -> &'static mut S { o.unwrap_or_else(|| Box::leak(Box::default())) }
fn main() {
    let b: Box<S> = Box::default();
    let c = Box::<u32>::default();
    let o: Option<Box<String>> = None;
    let d = o.unwrap_or_else(Box::default);
    let s = get(None);
    s.a = 3;
    println!("{:?} {} {:?} {:?}", b, c, d, s);
    KEEP.lock().unwrap().push(s);
}
