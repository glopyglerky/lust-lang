// expect: L0306
// The borrow a scoped thread returns through join must not outlive its data.
use std::thread;
fn main() {
    let r;
    {
        let v = vec![5, 6];
        r = thread::scope(|s| s.spawn(|| &v[0]).join().unwrap());
    }
    println!("{}", r);
}
