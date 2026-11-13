// expect: L0110
// `&A` with `A: &[u8; 1]` is a `&&[u8; 1]`, and no coercion reaches behind the outer reference.
const A: &[u8; 1] = &[1];
const B: &&[u8] = &A;
fn main() {
    println!("{:?}", B);
}
