// rustc-error: E0277
// lust-expect: L0612
// ARCH 2.5: a projection's trait ref is part of well-formedness,
// with the selected impl's where clauses: `Vec<String>: Tr` needs
// `String: Copy`.
trait Tr { type Out; }
impl<T: Copy> Tr for Vec<T> { type Out = T; }
fn f(x: <Vec<String> as Tr>::Out) -> usize { x.len() }
fn main() { println!("{}", f(String::new())); }
