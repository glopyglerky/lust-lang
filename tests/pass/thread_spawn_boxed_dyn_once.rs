// A `Box<dyn FnOnce() -> T + Send>` as a thread's function: `thread::spawn`,
// `Builder::spawn` (also as a fn value, gopanic.rs `GoThread::start`), and
// `scope.spawn`, with moved captures, a drop in the thread, a large result,
// and a body annotation whose object lifetime is inferred.
use std::thread;
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
#[derive(Debug)]
struct Big(u64, u64, u64, u64, u64, String);
fn start<H>(mut c: impl FnMut(thread::Builder, Box<dyn FnOnce() -> i32 + Send>) -> std::io::Result<H>) -> H {
    c(thread::Builder::new(), Box::new(|| 7)).unwrap()
}
fn main() {
    let h = start(thread::Builder::spawn);
    println!("{}", h.join().unwrap());
    let s = String::from("cap");
    let b: Box<dyn FnOnce() -> usize + Send> = Box::new(move || s.len());
    println!("{}", thread::spawn(b).join().unwrap());
    let d = D("moved");
    let b = Box::new(move || {
        let _k = &d;
        Big(1, 2, 3, 4, 5, "big".into())
    }) as Box<dyn FnOnce() -> Big + Send>;
    println!("{:?}", thread::Builder::new().name("w".into()).spawn(b).unwrap().join().unwrap());
    let v = vec![1, 2, 3];
    thread::scope(|sc| {
        let b: Box<dyn FnOnce() -> i32 + Send + '_> = Box::new(|| v.iter().sum());
        println!("{}", sc.spawn(b).join().unwrap());
    });
    let never = D("never run");
    let b: Box<dyn FnOnce() + Send> = Box::new(move || {
        let _k = &never;
    });
    drop(b);
    println!("end");
}
