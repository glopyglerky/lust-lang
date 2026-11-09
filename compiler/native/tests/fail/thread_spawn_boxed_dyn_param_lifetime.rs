// expect: L0733
// rustc E0521: the boxed function lives for the caller's `'a`, but
// `thread::spawn` needs `'static`.
fn f<'a>(b: Box<dyn FnOnce() -> i32 + Send + 'a>) -> i32 {
    std::thread::spawn(b).join().unwrap()
}
fn main() {
    let x = 5;
    println!("{}", f(Box::new(move || x)));
}
