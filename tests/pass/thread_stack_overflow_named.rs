// A stack overflow in a spawned thread: std gives each thread its own guard window and
// alternate signal stack, so the handler prints the thread's name ("<unknown>" when it has
// none) and aborts (134). tf-build: lust had the handler on the main thread only, and a
// spawned thread's overflow was a plain SIGSEGV (ts_goport's ipc conn tests).
// panic: has overflowed its stack
fn deep(n: u64, v: &mut Vec<u64>) -> u64 {
    let buf = [n; 64];
    v.push(buf[3]);
    if n == 0 { 0 } else { deep(n - 1, v) + buf[n as usize % 64] }
}

fn main() {
    println!("start");
    if std::env::args().count() == 1 {
        let r: std::io::Result<std::thread::JoinHandle<u64>> = std::thread::Builder::new().name("worker-7".into()).spawn(|| {
            let mut v = Vec::new();
            deep(u64::MAX, &mut v)
        });
        println!("{:?}", r.unwrap().join().is_ok());
    } else {
        let h = std::thread::spawn(|| {
            let mut v = Vec::new();
            deep(u64::MAX, &mut v)
        });
        println!("{:?}", h.join().is_ok());
    }
}
