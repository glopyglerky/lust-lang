fn main() {
    let x: u8 = 255;
    let y = x + std::env::args().count() as u8;
    println!("{y}");
}
