// expect: L0733
// rustc: lifetime may not live long enough. The `'static` output slot
// may not hold the captured `&'z u8`.
fn mk<'z>(z: &'z u8) -> impl Fn(&u8) -> &'static u8 + 'z {
    let f = move |a: &u8| z;
    f
}
fn main() {
    let z = 1u8;
    let f = mk(&z);
    let k = 2u8;
    println!("{}", f(&k));
}
