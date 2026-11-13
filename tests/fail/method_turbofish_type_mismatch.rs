// expect: L0110
// rustc E0308: x is u8 from the turbofish, then used as u32.
struct S<T>(T);
impl<T> S<T> { fn w<U: std::fmt::Debug>(&self, u: U) -> String { format!("{:?}", u) } }
fn main() { let x = 200; let y = S(1u8).w::<u8>(x); let z: u32 = x; println!("{} {}", y, z); }
