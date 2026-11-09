// ARCH 2.8: `Some(&x)` under an `Option<&dyn Tr>` expectation, and `Option<&dyn>` fields.
trait Ind { fn ind(&self) -> u32; }
struct D(u32);
impl Ind for D { fn ind(&self) -> u32 { self.0 } }
struct Cfg { cache: Option<&'static dyn Ind> }
static G: u32 = 9;
impl Ind for u32 { fn ind(&self) -> u32 { *self } }
fn run(i: Option<&dyn Ind>, extra: u32) -> u32 { match i { Some(d) => d.ind() + extra, None => extra } }
fn main() {
    let d = D(4);
    println!("{} {}", run(Some(&d), 1), run(None, 2));
    let o: Option<&dyn Ind> = Some(&d);
    let c = Cfg { cache: Some(&G) };
    println!("{} {}", o.unwrap().ind(), match c.cache { Some(x) => x.ind(), None => 0 });
}
