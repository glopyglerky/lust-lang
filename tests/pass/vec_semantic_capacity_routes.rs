// Growth policy uses the Rust element layout; storage keeps its C stride.
// The one-element operations expose the minimum, including the Darwin
// OnceLock boundary. Exact reservations and later growth are controls.
use std::sync::OnceLock;

#[derive(Clone)]
struct Envelope {
    value: OnceLock<[u8; 1020]>,
}

fn byte() -> u8 { 7 }
fn medium() -> [u8; 1024] { [7; 1024] }
fn large() -> [u8; 1025] { [9; 1025] }
fn once() -> OnceLock<[u8; 1020]> { OnceLock::new() }
fn wrapped() -> Envelope { Envelope { value: once() } }
fn optional() -> Option<OnceLock<[u8; 1012]>> { None }

fn show<T>(name: &str, route: &str, v: &Vec<T>) {
    println!("{} {} {} {}", name, route, v.len(), v.capacity());
}

fn routes<T: Clone>(name: &str, make: fn() -> T) {
    let collected: Vec<T> = (0..1).filter(|_| true).map(|_| make()).collect();
    show(name, "collect", &collected);

    let mut pushed = Vec::new();
    for _ in 0..5 {
        pushed.push(make());
        show(name, "push", &pushed);
    }

    let mut reserved: Vec<T> = Vec::new();
    reserved.reserve(1);
    show(name, "reserve", &reserved);
    reserved.reserve(1);
    show(name, "reserve-again", &reserved);

    let mut exact: Vec<T> = Vec::new();
    exact.reserve_exact(1);
    exact.push(make());
    show(name, "exact", &exact);
    exact.push(make());
    show(name, "exact-then-grow", &exact);

    let mut inserted = Vec::new();
    inserted.insert(0, make());
    show(name, "insert", &inserted);

    let mut source = vec![make()];
    let mut appended = Vec::new();
    appended.append(&mut source);
    show(name, "append", &appended);
    assert!(source.is_empty());

    let source = vec![make()];
    let mut extended = Vec::new();
    extended.extend_from_slice(&source);
    show(name, "extend-slice", &extended);

    let mut cloned = Vec::new();
    cloned.clone_from(&source);
    show(name, "clone-from", &cloned);

    let mut within = vec![make()];
    within.extend_from_within(..);
    show(name, "extend-within", &within);

    let mut resized = Vec::new();
    resized.resize(1, make());
    show(name, "resize", &resized);
    let mut resized_with = Vec::new();
    resized_with.resize_with(1, make);
    show(name, "resize-with", &resized_with);

    let mut spliced = Vec::new();
    spliced.splice(.., [make()]);
    show(name, "splice-empty", &spliced);
    let mut tail = vec![make()];
    tail.splice(0..0, [make()]);
    show(name, "splice-tail", &tail);
    let mut filtered = vec![make()];
    filtered.splice(0..0, (0..2).filter(|_| true).map(|_| make()));
    show(name, "splice-filtered", &filtered);
}

fn main() {
    routes("byte", byte);
    routes("medium", medium);
    routes("large", large);
    routes("once", once);
    routes("wrapped", wrapped);
    routes("optional", optional);
}
