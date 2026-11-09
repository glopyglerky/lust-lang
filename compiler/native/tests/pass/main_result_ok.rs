// main returning Result<(), E>: success path, with ? inside main.
fn work(x: i32) -> Result<i32, String> {
    if x < 0 { return Err("neg".to_string()); }
    Ok(x * 2)
}
fn main() -> Result<(), String> {
    let a = work(21)?;
    println!("{}", a);
    Ok(())
}
