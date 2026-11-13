// expect: L0110
// rustc E0053: the method's error type is not the impl's `Err`.
use std::str::FromStr;
struct N;
impl FromStr for N {
    type Err = String;
    fn from_str(s: &str) -> Result<N, i32> {
        let _ = s;
        Ok(N)
    }
}
fn main() {}
