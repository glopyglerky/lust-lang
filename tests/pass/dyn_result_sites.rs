// ARCH 2.8: `Ok(..)` and `Err(..)` under a `Result<&dyn Tr, _>` expectation are coercion sites,
// as `Some(..)` is under `Option<&dyn Tr>`.
trait T { fn v(&self) -> u8; }
struct S(u8);
impl T for S { fn v(&self) -> u8 { self.0 } }
fn pick(a: &S, ok: bool) -> Result<&dyn T, &dyn T> { if ok { Ok(a) } else { Err(a) } }
fn show(r: Result<&dyn T, u8>) -> u8 { match r { Ok(d) => d.v(), Err(e) => e } }
fn main() {
    let s = S(3);
    let r: Result<&dyn T, u8> = Ok(&s);
    let e: Result<u8, &dyn T> = Err(&s);
    let n: Result<(&dyn T, u8), u8> = Ok((&s, 1));
    if let Ok(d) = r { println!("{}", d.v()); }
    if let Err(d) = e { println!("{}", d.v()); }
    if let Ok((d, k)) = n { println!("{} {}", d.v(), k); }
    println!("{} {}", show(Ok(&s)), show(Err(9)));
    if let Err(d) = pick(&s, false) { println!("{}", d.v() + 1); }
}
