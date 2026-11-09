// expect: L0307
// rustc: the scoped thread borrows `x`, a local of the scope closure, but
// its handle leaves the closure (lifetime may not live long enough).
use std::thread;
fn main() {
    let _h = thread::scope(|sc| {
        let x = 5;
        thread::Builder::new().spawn_scoped(sc, || x + 1).unwrap()
    });
}
