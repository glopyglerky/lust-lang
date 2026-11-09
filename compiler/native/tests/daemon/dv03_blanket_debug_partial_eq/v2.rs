use std::fmt::Debug;
trait Show {
    fn show(&self) -> String;
}
impl<T: Debug + PartialEq> Show for T {
    fn show(&self) -> String {
        format!("{:?} {}", self, self == self)
    }
}
#[derive(Debug)]
struct A {
    x: i32,
}
#[derive(Debug, PartialEq)]
enum B {
    One,
    Two(A),
}
fn show_all<T: Show>(ts: &[T]) -> Vec<String> {
    ts.iter().map(|t| t.show()).collect()
}
fn main() {
    let a = A { x: 4 };
    println!("{}", a.show());
    println!("{:?}", show_all(&[B::One, B::Two(A { x: 1 })]));
}
