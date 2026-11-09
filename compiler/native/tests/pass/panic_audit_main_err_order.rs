// `main` returning Err prints "Error: .." before the held stdout tail.
fn main() -> Result<(), String> {
    println!("line");
    print!("tail");
    let n = std::env::args().count();
    if n > 0 {
        return Err(format!("bad {}", n));
    }
    Ok(())
}
