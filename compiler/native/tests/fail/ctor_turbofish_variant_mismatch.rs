// expect: L0110
// rustc E0308: args on the variant segment fix A = u8.
enum E<A> { One(A), Zero }
fn main() { let v = E::One::<u8>(5u16); if let E::One(x) = v { println!("{}", x); } let _z: E<u8> = E::Zero; }
