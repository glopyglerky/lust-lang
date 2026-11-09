// thread::scope: a joined panicking thread gives Err; an unjoined one
// makes the scope panic ("a scoped thread panicked") after all joins.
use std::thread;
fn main() {
    let mut v = vec![1, 2, 3];
    thread::scope(|s| {
        s.spawn(|| { println!("a"); }).join().unwrap();
        let h = s.spawn(|| -> i32 { panic!("joined panic") });
        println!("joined {}", h.join().is_err());
        v.push(4);
    });
    println!("{:?}", v);
    thread::scope(|s| {
        s.spawn(|| { panic!("unjoined"); });
    });
    println!("never");
}
