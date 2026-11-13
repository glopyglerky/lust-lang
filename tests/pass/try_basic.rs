// The ? operator on Option and Result, in fns and closures.
fn half(x: i32) -> Option<i32> { if x % 2 == 0 { Some(x / 2) } else { None } }
fn chain(x: i32) -> Option<i32> {
    let a = half(x)?;
    let b = half(a)?;
    Some(b + 1)
}
fn parse(s: &str) -> Result<i32, String> {
    if s == "7" { Ok(7) } else { Err("bad num".to_string()) }
}
fn run(s: &str) -> Result<i32, String> {
    let n = parse(s)?;
    Ok(n * 2)
}
fn main() {
    println!("{:?} {:?} {:?}", chain(16), chain(10), chain(3));
    println!("{:?} {:?}", run("7"), run("x"));
    let v: Option<i32> = None;
    let r = |u: Option<i32>| -> Option<i32> { let q = u?; Some(q + 1) };
    println!("{:?} {:?}", r(v), r(Some(4)));
}
