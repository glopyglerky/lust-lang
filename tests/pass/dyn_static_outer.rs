// `&'static dyn T` from static items, params, fields, and casts; it shortens to `&dyn T`.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
impl dyn T { fn more(&self) -> u32 { self.read() + 1 } }
static S0: S = S { n: 4 };
static S1: S = S { n: 9 };
struct H { x: &'static dyn T }
fn show(d: &'static dyn T) -> u32 { d.more() }
fn id(d: &'static dyn T) -> &'static dyn T { d }
fn keep(d: &'static dyn T) -> H { H { x: d } }
fn weak(d: &dyn T) -> u32 { d.read() }
fn main() {
    let h = H { x: &S0 };
    let d: &'static dyn T = &S1;
    let c = &S0 as &'static dyn T;
    let v: Vec<&'static dyn T> = vec![&S0, d];
    let w: Vec<&dyn T> = v;
    println!("{} {} {} {} {}", h.x.read(), show(&S0), id(d).read(), keep(c).x.more(), weak(d));
    println!("{} {}", w[0].read(), w[1].read());
}
