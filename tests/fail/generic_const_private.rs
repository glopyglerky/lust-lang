// expect: L0510
mod m {
    const SEC: i32 = 1;
}
fn g<T>(t: T) -> i32 {
    let _ = &t;
    m::SEC
}
fn main() {
    println!("{}", g(3));
}
