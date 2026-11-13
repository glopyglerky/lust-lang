// expect: L0105
fn main() {
    let x: Result<i32, String> = Ok(1, 2);
    println!("{:?}", x);
}
