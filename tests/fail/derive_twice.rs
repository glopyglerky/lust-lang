// expect: L0615
// rustc E0119: a trait derived twice (here across two attributes) is two
// conflicting impls.
#[derive(Debug, Clone)]
#[derive(Clone)]
struct S;
fn main() {
    println!("{:?}", S.clone());
}
