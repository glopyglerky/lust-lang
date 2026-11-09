// expect: L0303
// rustc E0502: the scoped thread borrows `v` mutably until the scope ends,
// so the read in the scope body conflicts.
use std::thread;
fn main() {
    let mut v = vec![1, 2, 3];
    thread::scope(|sc| {
        let h = thread::Builder::new().spawn_scoped(sc, || v.push(4)).unwrap();
        println!("{:?}", v);
        h.join().unwrap();
    });
}
