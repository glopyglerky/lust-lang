// expect: L0612
// Error::new needs an error payload; an integer is not one.
fn main() {
    let e = std::io::Error::new(std::io::ErrorKind::Other, 5);
    println!("{}", e);
}
