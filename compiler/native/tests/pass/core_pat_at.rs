fn main() {
    let x = 3;
    match x {
        v @ 0..=9 => println!("{}", v),
        _ => println!("big"),
    }
}
