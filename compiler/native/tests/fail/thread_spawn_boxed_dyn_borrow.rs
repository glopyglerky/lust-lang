// expect: L0306
// rustc E0597: the boxed function borrows `x`, but `thread::spawn` needs
// it to live for `'static` (the body annotation infers the object lifetime).
fn main() {
    let x = 5;
    let r = &x;
    let b: Box<dyn FnOnce() -> i32 + Send> = Box::new(move || *r);
    println!("{}", std::thread::spawn(b).join().unwrap());
}
