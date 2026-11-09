// expect: L0541
fn main() {
    let mut a = [1, 2, 3];
    a[5] = 9;
    println!("{}", a[0]);
}
