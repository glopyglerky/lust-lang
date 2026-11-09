// expect: L0630
// A constructor as a fn pointer in a const: the bounded const evaluator
// has no constructor fn values yet (L0630).
#[derive(Debug)]
struct J(i32);
const F: fn(i32) -> J = J;
fn main() {
    println!("{:?}", F(3));
}
