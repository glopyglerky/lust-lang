// rustc reborrows only the reference-typed `if`/`match` arms after the
// first one that does not diverge (LUB coercion). The first arm writes the
// result directly, so in statement position its dangling borrow of a tail
// temporary is never used. Blocks and non-reference arms are not
// reborrowed either.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
static ST: D = D("st");
fn mk(n: &'static str) -> D {
    println!("mk {}", n);
    D(n)
}
fn len(d: &D) -> usize {
    d.0.len()
}
fn first(c: bool) {
    if c { return } else { &mk("t2") };
    println!("after diverging first arm");
}
fn main() {
    let c = len(&D("c")) == 1;
    if c { &mk("t1") } else { &ST };
    let n = len(&D("n"));
    match n { 0 => { &mk("a") } _ => &ST };
    match n { 0 => &mk("a"), _ => &mk("b") };
    { &mk("t") };
    { { &mk("t") } };
    match &mk("t3") { r => r };
    if c { (&mk("t1"),) } else { (&mk("t2"),) };
    'a: { if c { break 'a &mk("t1"); } &mk("t2") };
    loop { if c { break &mk("t1"); } break { &mk("t2") }; };
    first(c);
    println!("end");
}
