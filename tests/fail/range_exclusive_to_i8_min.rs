// expect: L0119
fn main() {
    let x = 3i8;
    match x {
        ..i8::MIN => println!("empty"),
        _ => println!("other"),
    }
}
