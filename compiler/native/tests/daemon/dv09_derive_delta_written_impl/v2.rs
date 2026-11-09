// The derive delta (sigedit P3.2) next to a written impl: Clone moves
// from the derive list to a written impl and back. Adding the derive
// while the written impl stays is E0119; the delta refuses it
// ("derive written impl") and the full path answers.
const _ROUTE: u8 = 0;
#[derive(Debug, PartialEq)]
struct A {
    x: u8,
}
impl Clone for A {
    fn clone(&self) -> A {
        A { x: self.x + 1 }
    }
}
fn twice<T: Clone>(t: &T) -> (T, T) {
    (t.clone(), t.clone())
}
fn main() {
    let a = A { x: 3 };
    let (b, c) = twice(&a);
    println!("{:?} {}", b, b == c);
}
