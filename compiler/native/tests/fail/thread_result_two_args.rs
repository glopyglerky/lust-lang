// expect: L0110
// `std::thread::Result` takes one type argument (the error type is fixed).
fn f() -> std::thread::Result<u8, String> {
    Ok(1)
}
fn main() {
    println!("{:?}", f());
}
