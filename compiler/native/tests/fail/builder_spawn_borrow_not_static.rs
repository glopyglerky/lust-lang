// expect: L0306
// rustc E0597: `Builder::spawn` needs `F: 'static`; the closure borrows `x`.
fn main() {
    let x = 5;
    let r = &x;
    let h = std::thread::Builder::new().spawn(|| *r).unwrap();
    println!("{:?}", h.join().unwrap());
}
