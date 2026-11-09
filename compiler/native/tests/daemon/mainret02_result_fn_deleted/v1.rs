fn pad() -> u32 { 5 }
fn other() -> String { String::from("from other") }
fn main() -> Result<(), String> { println!("main {}", pad()); let _ = other(); Ok(()) }
