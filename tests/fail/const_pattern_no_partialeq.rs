// expect: L0633
// The const's type must implement PartialEq, even for a `None` value.
struct NoEq;
const C: Option<NoEq> = None;
fn main() {
    let x: Option<NoEq> = None;
    match x {
        C => println!("a"),
        _ => println!("b"),
    }
}
