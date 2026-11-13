// expect: L0633
// Function pointers cannot be used in patterns.
fn f() {}
const C: fn() = f;
fn main() {
    let x: fn() = f;
    match x {
        C => println!("a"),
        _ => println!("b"),
    }
}
