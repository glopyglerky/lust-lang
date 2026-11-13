// expect: L0106
fn main() {
    match (1, 2) {
        (x, x) => println!("{}", x),
    }
}
