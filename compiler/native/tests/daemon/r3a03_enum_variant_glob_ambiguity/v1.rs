mod b {
    pub const B: u32 = 7;
}
enum E {
    A,
}
use E::*;
use b::*;
fn main() {
    let _e = A;
    println!("{}", B);
}
