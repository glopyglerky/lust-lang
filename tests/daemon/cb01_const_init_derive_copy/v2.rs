#[derive(Clone)]
struct K {
    x: u32,
}
const P: (K, K) = {
    let k = K { x: 2 };
    (k, k)
};
fn main() {
    println!("{}", P.0.x + P.1.x);
}
