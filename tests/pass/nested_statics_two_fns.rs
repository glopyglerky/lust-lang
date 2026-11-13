// Two statics inside function bodies, one name in two fns: each is its
// own static (they once shared one C symbol).
fn a() -> i32 {
    static X: i32 = 1;
    X
}
fn b() -> i32 {
    static X: i32 = 2;
    static Y: &str = "y";
    X + Y.len() as i32
}
fn main() {
    println!("{} {}", a(), b());
}
