// expect: L0306
// rustc E0597: a call through `dyn Fn(&u8) -> &u8` returns a borrow of its argument.
fn main() {
    let f: &dyn Fn(&u8) -> &u8 = &|x| x;
    let r;
    {
        let k = 1u8;
        r = f(&k);
    }
    println!("{}", r);
}
