// A thread function as a `Box<dyn FnOnce>` through `Builder::spawn` as a
// fn value (gopanic.rs `GoThread::start`): prints 7.
fn start<H>(mut c: impl FnMut(std::thread::Builder, Box<dyn FnOnce() -> i32 + Send>) -> std::io::Result<H>) -> H {
    c(std::thread::Builder::new(), Box::new(|| 7)).unwrap()
}
fn main() {
    let h = start(std::thread::Builder::spawn);
    println!("{}", h.join().unwrap());
}
