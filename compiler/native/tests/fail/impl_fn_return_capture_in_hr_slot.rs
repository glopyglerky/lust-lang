// expect: L0733
// rustc: lifetime may not live long enough. The output is the caller's
// argument lifetime, any one, so a captured `&'z u8` cannot be it.
fn mk<'z>(z: &'z u8) -> impl Fn(&u8) -> &u8 + 'z {
    move |_a| z
}
fn main() {
    let z = 1u8;
    let f = mk(&z);
    let k = 2u8;
    println!("{}", f(&k));
}
