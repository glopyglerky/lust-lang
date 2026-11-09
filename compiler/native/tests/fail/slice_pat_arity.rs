// expect: L0110
fn main() {
    let a = [1, 2, 3];
    match a {
        [x, y] => println!("{} {}", x, y),
        _ => println!("other"),
    }
}
