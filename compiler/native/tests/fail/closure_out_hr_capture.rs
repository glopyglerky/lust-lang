// expect: L0306
// rustc E0597: under `Fn(&u8) -> &u8` the output is the argument's higher-ranked lifetime, which the callee picks; a captured borrow returned there must be `'static`.
fn app<F: Fn(&u8) -> &u8>(f: F) -> u8 {
    let x = 5u8;
    *f(&x)
}

fn main() {
    let k = 7u8;
    println!("{}", app(|_| &k));
}
