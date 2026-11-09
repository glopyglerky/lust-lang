// expect: L0612
// rustc E0277: `N` is not `Clone`. `lust build` used to call the mono
// instance of `m` directly once it existed, skipping the bound.
struct N;
struct Z;
impl Z {
    fn m<X: Clone>(&self, _x: X) -> u32 {
        1
    }
}
fn main() {
    println!("{}", Z.m(N));
}
