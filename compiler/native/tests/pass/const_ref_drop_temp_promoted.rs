// A promoted const borrow's pointee is never dropped.
struct D(u8);
impl Drop for D { fn drop(&mut self) { println!("drop"); } }
const S: &D = &D(1);
const T: &Option<D> = &Some(D(2));
fn main() { let _ = S; let _ = T; println!("ok {} {}", S.0, T.as_ref().map_or(0, |d| d.0)); }
