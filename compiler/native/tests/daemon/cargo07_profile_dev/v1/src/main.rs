fn main() {
    let n = std::env::args().count() as u8;
    debug_assert!(n == 2, "debug assertion");
    let x: u8 = 254 + n;
    println!("start");
    println!("{}", x + 1);
}
