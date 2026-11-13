// expect: L0001
fn main() {
    let x = [5u8];
    match x {
        [0] => println!("zero"),
        [1..] => println!("more"),
        _ => println!("other"),
    }
}
