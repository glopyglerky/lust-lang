fn other() -> String { String::from("from other") }
fn main() -> Result<(), String> { println!("main"); let _ = other(); Ok(()) }
