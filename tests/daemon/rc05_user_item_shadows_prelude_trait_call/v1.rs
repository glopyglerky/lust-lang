fn get() -> i32 {
    let x: i32 = Default::default();
    x + 1
}
fn main() {
    println!("{}", get());
}
