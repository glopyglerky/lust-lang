// expect: L0110
fn main() {
    let x: i32 = 1;
    println!("{}", matches!(x, 1.0));
}
