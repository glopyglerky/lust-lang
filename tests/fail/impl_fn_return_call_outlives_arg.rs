// expect: L0306
// rustc E0597: the result of a returned `impl Fn(&u8) -> &u8` borrows
// the call's argument.
fn mk() -> impl Fn(&u8) -> (&u8, u8) {
    |a| (a, 1)
}
fn main() {
    let f = mk();
    let r;
    {
        let k = 2u8;
        r = f(&k);
    }
    println!("{:?}", r);
}
