fn get() -> i32 {
    let f: fn() -> i32 = Default::default;
    f() + 1
}
fn main() {
    println!("{}", get());
}
