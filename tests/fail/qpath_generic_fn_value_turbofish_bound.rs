// expect: L0612
// rustc E0277: the turbofish type i8 has no From<u8>.
trait Conv { fn conv<U: From<u8>>(&self) -> U; }
struct S(u8);
impl Conv for S { fn conv<U: From<u8>>(&self) -> U { U::from(self.0) } }
fn main() { let f = <S as Conv>::conv::<i8>; println!("{}", f(&S(7))); }
