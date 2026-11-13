// Calls through Arc, Rc and Box of a closure or an Fn-bounded parameter (ts-rust program.rs:5400).
use std::rc::Rc;
use std::sync::Arc;
fn run<T, F: Fn(u32) -> T + Send + Sync + 'static>(files: &[u32], f: F) -> Vec<T> {
    let f = Arc::new(f);
    files.iter().map(|&file| { let f = Arc::clone(&f); let g = move || f(file); g() }).collect()
}
fn rc_call<F: Fn() -> u8>(f: Rc<F>) -> u8 { f() + f() }
fn box_call<F: FnMut() -> u8>(mut f: Box<F>) -> u8 { f() + f() }
fn main() {
    println!("{:?}", run(&[1, 2], |x| x * 10));
    let c = Arc::new(|x: u8| x + 1);
    println!("{}", c(4));
    println!("{}", rc_call(Rc::new(|| 3)));
    let mut n = 0;
    println!("{} {}", box_call(Box::new(|| { n += 1; n })), n);
}
