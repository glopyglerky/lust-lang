// expect: L0119
fn main() {
    let x = 3;
    match x {
        5..5 => println!("a"),
        _ => println!("b"),
    }
}
