// expect: L0110
// rustc E0530: match bindings cannot shadow tuple variants.
fn main() {
    let x: Option<i32> = Some(1);
    match x {
        Some => println!("a"),
        None => println!("b"),
    }
}
