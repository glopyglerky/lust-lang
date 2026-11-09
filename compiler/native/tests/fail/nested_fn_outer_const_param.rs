// expect: L0101
// rustc E0401: a nested fn cannot use the outer fn's const param `N`, and
// the param still hides the module const `N` (lust took the const).
const N: usize = 10;
fn f<const N: usize>() -> usize {
    fn g() -> usize {
        N
    }
    g() + N
}
fn main() {
    println!("{}", f::<3>());
}
