// expect: L0112
struct E1;
struct E2;
fn inner() -> Result<i32, E1> { Err(E1) }
fn outer() -> Result<i32, E2> {
    let v = inner()?;
    Ok(v)
}
fn main() { match outer() { Ok(v) => println!("{}", v), Err(_) => println!("err") } }
