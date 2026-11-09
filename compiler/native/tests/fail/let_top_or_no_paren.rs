// expect: L0001
fn main() {
    let (x, _) | (_, x) = (7, 8);
    println!("{}", x);
}
