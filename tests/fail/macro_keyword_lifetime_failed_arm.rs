// expect: L0001
// rustc: the first arm's `ty` parse reports `'fn` before the arm fails at `y`;
// the second arm then matches, but the error stands.
macro_rules! m { ($t:ty ; x) => { 1 }; ($($x:tt)*) => { 2 } }
fn main() { println!("{}", m!(&'fn u8 ; y)); }
