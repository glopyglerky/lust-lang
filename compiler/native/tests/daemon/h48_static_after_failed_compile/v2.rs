const fn seed_val() -> i64 { 2 }
const SEED: i64 = seed_val();
static TABLE: [i64; 2] = [SEED, SEED + 1];
fn main() { println!("{}", TABLE[1]); }
