// expect: L0101
macro_rules! m { () => { let secret = 3; } }
fn main() { m!(); println!("{}", secret); }
