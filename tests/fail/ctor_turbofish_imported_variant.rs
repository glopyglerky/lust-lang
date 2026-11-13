// expect: L0110
// rustc E0308: args on an imported variant fix A = u8.
enum E<A> { One(A), Zero }
use E::*;
fn main() { let v = One::<u8>(250); let y: u16 = match v { One(x) => x, Zero => 0 }; println!("{}", y); }
