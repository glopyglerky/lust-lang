// Each thread has its own thread_local! values: a spawned thread starts
// from the initializer, its changes stay its own, and its destructors run
// when it ends (before join returns). Main's run at process exit.
use std::cell::{Cell, RefCell};
use std::thread;
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D({})", self.0);
    }
}
thread_local! {
    static C: Cell<u32> = const { Cell::new(0) };
    static G: D = D(1);
    static V: RefCell<Vec<u32>> = RefCell::new(Vec::new());
}
fn main() {
    C.set(5);
    V.with_borrow_mut(|v| v.push(1));
    let h = thread::spawn(|| {
        C.set(C.get() + 1);
        G.with(|d| println!("child sees D({})", d.0));
        V.with_borrow_mut(|v| v.push(7));
        V.with_borrow(|v| println!("child vec {:?}", v));
        C.get()
    });
    let v = h.join().unwrap();
    println!("child {} main {}", v, C.get());
    V.with_borrow(|v| println!("main vec {:?}", v));
    let hs: Vec<_> = (0..3u32).map(|i| thread::spawn(move || { C.set(i * 10); C.get() })).collect();
    let mut rs: Vec<u32> = hs.into_iter().map(|h| h.join().unwrap()).collect();
    rs.sort();
    println!("{:?} main still {}", rs, C.get());
    G.with(|d| println!("main sees D({})", d.0));
}
