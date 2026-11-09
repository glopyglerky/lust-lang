// expect: L0115
// A struct const pattern covers one value, not the type.
#[derive(PartialEq, Eq)]
struct C(u8);
const A: C = C(1);
fn main() {
    match C(2) {
        A => println!("a"),
    }
}
