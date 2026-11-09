// expect: L0110
static S: i32 = 3;
fn f(x: i32) -> i32 {
    match x {
        S => 1,
        _ => 0,
    }
}
fn main() {
    println!("{}", f(3));
}
