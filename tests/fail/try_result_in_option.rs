// expect: L0112
fn f() -> Option<i32> {
    let x: Result<i32, String> = Ok(1);
    let v = x?;
    Some(v)
}
fn main() { println!("{:?}", f()); }
