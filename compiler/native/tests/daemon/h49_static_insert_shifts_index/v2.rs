static FIRST: [i64; 2] = [11, 12];
static TABLE: [i64; 2] = [3, 4];
fn get() -> i64 { TABLE[1] }
fn main() { println!("{} {}", get(), FIRST[1]); }
