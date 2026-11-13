// `thread::Builder` (name, stack size, spawn) and
// `thread::available_parallelism` (TRIAGE T5). Only stable
// facts print: the worker's name, its return, and that at
// least one CPU is available.
use std::thread::Builder;

fn read<T>(v: T) -> T {
    v
}

fn main() {
    println!("{:?}", Builder::new());
    let h = Builder::new()
        .name("worker".to_string())
        .stack_size(1 << 20)
        .spawn(|| {
            println!("in {:?}", std::thread::current().name());
            read(42)
        })
        .unwrap();
    println!("{}", h.join().unwrap());
    let h2 = Builder::new().spawn(|| 7).unwrap();
    println!("{}", h2.join().unwrap());
    match std::thread::available_parallelism() {
        Ok(n) => println!("cpus>=1 {}", n.get() >= 1),
        Err(e) => println!("err {:?}", e.kind()),
    }
}
