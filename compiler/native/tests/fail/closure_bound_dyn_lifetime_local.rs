// expect: L0306
// rustc E0597: the closure moves the box into a thread, so the callee's `'a` must be
// `'static`, and `x` borrows a local.
fn local<'a>(x: &'a i32, mut clone: impl FnMut(Box<dyn Fn() -> i32 + Send + Sync + 'a>) -> i32) -> i32 {
    clone(Box::new(move || *x))
}

fn main() {
    let n = 4;
    let x = &n;
    println!("{}", local(x, |b| std::thread::spawn(move || b()).join().unwrap()));
}
