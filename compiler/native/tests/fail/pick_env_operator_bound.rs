// expect: L0612
// rustc E0277: M has PartialEq<u8>, the bound asks for PartialEq<u16>.
#[derive(Clone, Copy)]
struct M(u32);
impl PartialEq<u8> for M { fn eq(&self, o: &u8) -> bool { self.0 == *o as u32 } }
fn f<T: PartialEq<u16>>(t: T) -> bool { t == 5 }
fn main() { println!("{}", f(M(5))); }
