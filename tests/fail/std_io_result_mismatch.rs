// expect: L0110
// io::Result<T> is Result<T, io::Error>.
fn main() {
    let r: std::io::Result<i32> = Ok(1);
    let x: Result<i32, String> = r;
    println!("{:?}", x);
}
