// expect: L0110
// A const pattern never auto-derefs the scrutinee, even when its own type is a reference.
const C: &u8 = &1;
fn main() {
    let x: &&u8 = &&1;
    match x {
        C => println!("1"),
        _ => println!("2"),
    }
}
