// ARCH 2.8: `+ Send`, `+ Sync`, auto traits from supertraits, and dropping auto traits.
trait Tr { fn v(&self) -> u8; }
trait SendTr: Send { fn w(&self) -> u8; }
struct A(u8);
impl Tr for A { fn v(&self) -> u8 { self.0 } }
impl SendTr for A { fn w(&self) -> u8 { self.0 + 1 } }
fn need_send<T: Send + ?Sized>(_x: &T) -> u8 { 1 }
fn need_sync<T: Sync + ?Sized>(_x: &T) -> u8 { 2 }
fn main() {
    let a = A(4);
    let s: &(dyn Tr + Send) = &a;
    let ss: &(dyn Tr + Send + Sync) = &a;
    let plain: &dyn Tr = s;
    let st: &dyn SendTr = &a;
    println!("{} {} {} {}", s.v(), ss.v(), plain.v(), st.w());
    println!("{} {} {}", need_send(s), need_sync(ss), need_send(st));
    let st2: &(dyn SendTr + Send) = st;
    println!("{}", st2.w());
}
