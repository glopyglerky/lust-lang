fn main() {
    let x = 3;
    match x {
        0..10 => println!("small"),
        _ => println!("big"),
    }
}
