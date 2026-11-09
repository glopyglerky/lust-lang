// `Builder::spawn_scoped(scope, f)` (ts-rust program.rs, emit_files.rs via
// gopanic.rs `GoThread::spawn_scoped`): a named thread that borrows from the
// enclosing frame, one that mutates a local, and a boxed `dyn FnOnce`.
use std::thread;
fn main() {
    let v = vec![1, 2, 3];
    let mut m = 0;
    let s = thread::scope(|sc| {
        let h = thread::Builder::new()
            .name("sw".into())
            .spawn_scoped(sc, || {
                println!("in {:?}", thread::current().name());
                v.iter().sum::<i32>()
            })
            .unwrap();
        let h2 = thread::Builder::new().stack_size(1 << 20).spawn_scoped(sc, || {
            m += 1;
            m
        });
        h.join().unwrap() + h2.unwrap().join().unwrap()
    });
    println!("{s} {m} {:?}", v);
    let b: Box<dyn FnOnce() -> usize + Send + '_> = Box::new(|| v.len());
    thread::scope(|sc| println!("{}", thread::Builder::new().spawn_scoped(sc, b).unwrap().join().unwrap()));
}
