// expect: L0104
// rustc E0599 only: once the body has an error, rustc reports no E0282
// for the closure parameter the failed call leaves unknown.
fn main() {
    let v = vec![1u8, 2];
    let n = v.no_such_method(|x| x.count_ones());
    println!("{}", n);
}
