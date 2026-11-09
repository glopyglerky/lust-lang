fn main() {
    let x: u8 = 254 + std::env::args().count() as u8;
    println!("start");
    println!("{}", x + 1);
}
