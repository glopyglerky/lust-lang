const fn init() -> u32 { 7 }
static S: u32 = init();
static R: &u32 = &S;
fn main() { println!("{}", R); }
