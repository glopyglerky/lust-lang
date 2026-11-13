// A closure returns a handle that holds its captured `&Scope`'s lifetime:
// the return needs that capture, which outlives the closure's env borrow
// (an implied bound), so it does not need the env borrow itself
// (ts-rust gopanic.rs GoThread::spawn_scoped).
fn start<'a, T: 'a, H>(f: impl FnOnce() -> T + Send + 'a, mut clone: impl FnMut(std::thread::Builder, Box<dyn FnOnce() -> T + Send + 'a>) -> std::io::Result<H>) -> H {
    clone(std::thread::Builder::new(), Box::new(f)).unwrap()
}
fn main() {
    std::thread::scope(|scope| {
        let h = start(|| 7, |builder, run| builder.spawn_scoped(scope, run));
        println!("{}", h.join().unwrap());
        let h = call(|| scope.spawn(|| 8));
        println!("{}", h.join().unwrap());
    });
}

fn call<H>(mut f: impl FnMut() -> H) -> H {
    f()
}
