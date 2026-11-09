// expect: L0110
// rustc E0277: deriving `Ord` requires every field to implement `Ord`.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum E { A((u8, Vec<f64>)), B }
fn main() { let _ = E::A((1u8, vec![1.0f64])); }
