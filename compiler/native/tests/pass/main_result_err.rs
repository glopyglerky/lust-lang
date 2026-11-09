// panic: Error: "negative"
// main returning Result<(), E>: Err prints `Error: {err:?}` to
// stderr and exits 1 (Rust Termination), after ? propagates.
fn work(x: i32) -> Result<(), String> {
    if x < 0 { return Err("negative".to_string()); }
    println!("did {}", x);
    Ok(())
}
fn main() -> Result<(), String> {
    work(3)?;
    work(-1)?;
    println!("unreachable");
    Ok(())
}
