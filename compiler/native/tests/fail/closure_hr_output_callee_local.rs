// expect: L0307
// rustc E0515: under `Fn(&u8) -> &u8` the call's result borrows the callee's local `x`.
fn app<F: Fn(&u8) -> &u8>(f: F) -> &'static u8 {
    let x = 5u8;
    f(&x)
}

fn main() {
    println!("{}", app(|x| x));
}
