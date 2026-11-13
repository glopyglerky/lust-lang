fn w<T>(_x: &T) -> &'static str where Vec<T>: Send { "wsend" }
fn main() { println!("{}", w(&1u8)); }
