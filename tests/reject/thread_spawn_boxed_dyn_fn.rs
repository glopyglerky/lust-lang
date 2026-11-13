// expect: L0702
// Valid Rust (prints 1): a `Box<dyn Fn>` as a thread's function. lust's
// thread start calls a boxed `dyn FnOnce` only.
fn main() {
    let b: Box<dyn Fn() -> i32 + Send> = Box::new(|| 1);
    println!("{}", std::thread::spawn(b).join().unwrap());
}
