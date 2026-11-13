// expect: L0303
// rustc E0499 (lust reports the shared reservation: L0303): the reservation of `x` cannot be written by a later argument.
fn f(v: &mut Vec<u8>, n: usize) {
    v.push(n as u8);
}
fn main() {
    let mut lv = vec![1u8];
    let x = &mut lv;
    f(x, {
        x.push(1);
        0
    });
    println!("{:?}", lv);
}
