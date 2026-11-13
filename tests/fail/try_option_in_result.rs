// expect: L0112
fn f() -> Result<i32, String> {
    let x: Option<i32> = Some(1);
    let v = x?;
    Ok(v)
}
fn main() { println!("{:?}", f()); }
