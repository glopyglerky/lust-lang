const N: usize = 3;
#[allow(dead_code)]
struct N {}
fn f<const K: usize>() -> usize {
    K * 2
}
fn get() -> usize {
    f::<N>()
}
fn main() {
    println!("{}", get());
}
