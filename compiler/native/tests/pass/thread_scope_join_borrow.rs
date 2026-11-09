// A scoped thread returns a borrow of the scope's environment through join (the
// join's Ok carries the thread's T only; its Err payload borrows nothing).
use std::thread;
fn main() {
    let v = vec![5, 6];
    let r = thread::scope(|s| s.spawn(|| &v[0]).join().unwrap());
    println!("{}", r);
}
