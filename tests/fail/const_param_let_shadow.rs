// expect: L0110
// rustc E0530: a `let mut` binding cannot shadow a const param.
fn f<const N: usize>(o: usize) -> usize {
    let mut N = o;
    N += 1;
    N
}
fn main() {
    println!("{}", f::<3>(3));
}
