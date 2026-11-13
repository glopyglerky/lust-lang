// Array lengths read same-name inherent consts of disjoint impls
// through the typed pick, as rustc does.
struct G<T>(T);
impl G<u8> {
    const N: usize = 3;
}
impl G<i32> {
    const N: usize = 5;
}
struct H<T>(T);
impl<T> H<T> {
    const M: usize = 4;
}
struct One<T>(T);
impl One<u8> {
    const K: usize = 2;
}
fn one() -> [u8; One::K] {
    [9; One::K]
}
fn main() {
    let a = [0u8; G::<u8>::N];
    let b = [1u8; G::<i32>::N];
    let c = [2u8; H::<bool>::M];
    let d = [3u8; One::K];
    println!("{} {} {} {} {}", a.len(), b.len(), c.len(), d.len(), one().len());
    println!("{} {}", G::<i32>::N + G::<u8>::N, std::mem::size_of_val(&b));
}
