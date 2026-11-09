// expect: L0110
// rustc E0158: a const param cannot be a pattern (lust matched every value).
fn f<const N: usize>(o: usize) -> usize {
    match o {
        N => 100,
        x => x,
    }
}
fn main() {
    println!("{} {}", f::<3>(3), f::<3>(4));
}
