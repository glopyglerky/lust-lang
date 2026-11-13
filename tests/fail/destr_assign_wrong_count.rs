// expect: L0110
fn main() {
    let mut a = 0;
    let mut b = 0;
    let mut c = 0;
    (a, b, c, ..) = (1, 2);
    println!("{} {} {}", a, b, c);
}
