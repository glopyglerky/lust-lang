// expect: L0306
// rustc E0597: under `dyn Fn(&u8) -> &u8` the closure's result has its argument's late-bound lifetime, so returning a captured borrow needs `'static`.
fn main() {
    let k = 3u8;
    let f: Box<dyn Fn(&u8) -> &u8> = Box::new(|_| &k);
    let x = 1u8;
    println!("{}", f(&x));
}
