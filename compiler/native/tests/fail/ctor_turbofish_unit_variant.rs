// expect: L0110
// rustc E0308: a unit variant with written args has that type.
enum E<A> { One(A), Zero }
fn main() { let z = E::<u8>::Zero; let o = E::One(3u16); let v = vec![z, o]; println!("{}", v.len()); }
