// expect: L0119
fn main() {
    let x = 3u8;
    match x {
        ..0 => println!("empty"),
        _ => println!("other"),
    }
}
