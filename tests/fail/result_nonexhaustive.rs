// expect: L0115
fn main() {
    let x: Result<i32, String> = Ok(1);
    match x {
        Ok(v) => println!("{}", v),
    }
}
