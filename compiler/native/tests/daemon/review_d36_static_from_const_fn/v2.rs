const fn init() -> u32 { 70 }
static S: u32 = init();
fn get() -> u32 { S }
fn main() { println!("{}", get()); }
