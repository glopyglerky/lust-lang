// expect: L0540
fn main() {
    let x: i32 = i32::MIN;
    let y = -x;
    println!("{}", y);
}
