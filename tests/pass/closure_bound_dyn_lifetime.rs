// A closure (or a fn value) for `impl FnMut(Box<dyn FnOnce() -> T + Send + 'a>)`:
// the callee's `'a` is a region of the call, so a closure that needs the box to be
// `'static` (a thread spawn) makes `'a` `'static` there, as rustc infers
// (ts-rust gopanic.rs GoThread::spawn with `std::thread::Builder::spawn`).
fn start<'a, F, T, H>(f: F, mut clone: impl FnMut(std::thread::Builder, Box<dyn FnOnce() -> T + Send + 'a>) -> std::io::Result<H>) -> H
where
    F: FnOnce() -> T + Send + 'a,
    T: 'a,
{
    clone(std::thread::Builder::new(), Box::new(f)).unwrap()
}

fn spawn<F, T>(f: F) -> std::thread::JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    start(f, std::thread::Builder::spawn)
}

fn local<'a>(x: &'a i32, mut clone: impl FnMut(Box<dyn Fn() -> i32 + Send + Sync + 'a>) -> i32) -> i32 {
    clone(Box::new(move || *x))
}

fn main() {
    let s = String::from("thread");
    println!("{}", spawn(move || s.len()).join().unwrap());
    static N: i32 = 4;
    println!("{}", local(&N, |b| std::thread::spawn(move || b()).join().unwrap()));
    let n = 5;
    println!("{}", local(&n, |b| b() + 1));
}
