const N: usize = 3;
fn f<const K: usize>() -> usize {
    K * 2
}
fn get() -> usize {
    f::<N>()
}
fn main() {
    println!("{}", get());
}
