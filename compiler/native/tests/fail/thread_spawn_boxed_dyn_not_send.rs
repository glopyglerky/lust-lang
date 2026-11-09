// expect: L0612
// rustc E0277: `dyn FnOnce() -> i32` is not `Send`.
fn main() {
    let b: Box<dyn FnOnce() -> i32> = Box::new(|| 1);
    println!("{}", std::thread::spawn(b).join().unwrap());
}
