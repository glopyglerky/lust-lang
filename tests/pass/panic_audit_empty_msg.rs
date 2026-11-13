// `panic!("")` prints an empty message, not "explicit panic".
fn main() {
    let n = std::env::args().count();
    println!("{}", n);
    if n > 0 {
        panic!("");
    }
}
