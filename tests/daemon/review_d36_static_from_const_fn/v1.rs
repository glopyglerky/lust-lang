const fn init() -> u32 { 7 }
static S: u32 = init();
fn get() -> u32 { S }
fn main() { println!("{}", get()); }
