struct Default;
impl Default {
    fn default() -> i32 {
        7
    }
}
fn get() -> i32 {
    let x: i32 = Default::default();
    x + 1
}
fn main() {
    let _ = Default;
    println!("{}", get());
}
