struct Default;
impl Default {
    fn default() -> i32 {
        7
    }
}
fn get() -> i32 {
    let f: fn() -> i32 = Default::default;
    f() + 1
}
fn main() {
    let _ = Default;
    println!("{}", get());
}
