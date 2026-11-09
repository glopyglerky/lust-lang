// expect: L0110
fn main() {
    let mut x = 0;
    let mut y = 0;
    let s: &[i32] = &[1, 2, 3];
    [x, .., y] = *s;
    println!("{} {}", x, y);
}
