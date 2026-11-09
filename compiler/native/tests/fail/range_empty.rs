// expect: L0119
fn main() {
    let x: i64 = 3;
    match x {
        5..=1 => println!("empty"),
        _ => println!("other"),
    }
}
