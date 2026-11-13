// expect: L0303
// rustc E0502: an explicit `&mut` argument is not two-phase.
fn f(v: &mut Vec<u8>, n: usize) {
    v.push(n as u8);
}
fn main() {
    let mut lv = vec![1u8];
    f(&mut lv, lv.len());
    println!("{:?}", lv);
}
