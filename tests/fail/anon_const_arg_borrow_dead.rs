// expect: L0303
// rustc E0502: a const generic argument in a signature is borrow-checked as a body.
struct W<const N: usize>;
impl<const N: usize> W<N> {
    fn n(&self) -> usize {
        N
    }
}
type A = W<{
    if false {
        let mut x = 1;
        let r = &mut x;
        let r2 = &x;
        *r = 2;
        let _z = *r2;
    }
    3
}>;
fn main() {
    let w: A = W;
    println!("{}", w.n());
}
