// ARCH 2.8: statics behind `&'static dyn` fields and `dyn Tr + 'static` params.
trait Tr { fn get(&self) -> u32; }
impl Tr for u32 { fn get(&self) -> u32 { *self * 2 } }
impl Tr for u8 { fn get(&self) -> u32 { *self as u32 } }
static ONE: u32 = 21;
static BYTE: u8 = 7;
struct Holder { t: &'static dyn Tr, n: u32 }
fn keep(d: &(dyn Tr + 'static)) -> u32 { d.get() }
fn main() {
    let h = Holder { t: &ONE, n: 1 };
    println!("{} {}", h.t.get(), h.n);
    println!("{} {}", keep(&ONE), keep(&BYTE));
    let d: &dyn Tr = &BYTE;
    println!("{}", d.get() + h.t.get());
}
