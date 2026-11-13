// expect: L0001
fn main() {
    let c = &5u8;
    match c {
        &0..=9 => println!("digit"),
        _ => println!("other"),
    }
}
