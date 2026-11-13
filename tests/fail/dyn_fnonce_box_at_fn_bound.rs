// expect: L0612
// rustc E0277: `Box<dyn FnOnce()>` is only `FnOnce`.
fn apply<F: Fn() -> u8>(f: F) -> u8 {
    f()
}
fn main() {
    let b: Box<dyn FnOnce() -> u8> = Box::new(|| 3);
    println!("{}", apply(b));
}
