trait Tr { fn m(&self) -> u32; }
trait Third { fn t(&self) -> u32; }
struct S(u32);
struct P;
impl Tr for S { fn m(&self) -> u32 { self.0 } }
impl Third for P { fn t(&self) -> u32 { 9 } }
const CS: S = S(4);
static CP: P = P;
fn visit(f: &mut dyn FnMut(u32) -> u32, x: u32) -> u32 where u32: Copy { f(x) }
fn show(t: &(dyn Tr + Send)) -> u32 { t.m() }
fn opt(_t: Option<u8>) -> u32 { 0 }
fn usec() -> u32 { let d: &dyn Tr = &CS; d.m() }
fn uses_p() -> u32 { let p: &P = &CP; p.t() }
fn wrap<T: Copy>(t: T) -> T { t }
fn main() {
    let mut n = 0;
    let r = visit(&mut |x| { n += x; x * 2 }, 3);
    println!("{} {} {} {} {} {} {}", r, n, show(&S(5)), opt(None), usec(), uses_p(), wrap(1u8));
}
