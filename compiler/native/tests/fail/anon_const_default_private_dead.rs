// expect: L0510
// rustc E0624: a const param default's body sees privacy, also in a dead branch.
mod m {
    pub struct S;
    impl S {
        const N: usize = 1;
    }
}
struct W<const N: usize = {
    if false {
        let _n = m::S::N;
    }
    3
}>;
impl<const N: usize> W<N> {
    fn n(&self) -> usize {
        N
    }
}
fn main() {
    let w: W = W;
    println!("{}", w.n());
}
