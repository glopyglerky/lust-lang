// An expression statement's value goes into a statement temporary, as in
// rustc: `s;` moves `s` and drops it there, and a block, `if`, `match`,
// or `loop` value drops at the end of the statement, after the locals of
// the block that made it.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
struct P {
    a: D,
    b: D,
}
fn mk(n: &'static str) -> D {
    println!("mk {}", n);
    D(n)
}
fn len(d: &D) -> usize {
    d.0.len()
}
fn main() {
    { let _y = mk("y"); mk("t") };
    println!("block");
    let _ = { let _y = mk("y"); mk("t") };
    println!("wild block");
    match 1 { _ => { let _y = mk("y"); mk("t") } };
    println!("match");
    let c = len(&D("c")) == 1;
    if c { let _y = mk("y"); mk("t") } else { mk("u") };
    println!("if");
    let o = Some(1);
    if let Some(_v) = o { let _y = mk("y"); mk("t") } else { mk("u") };
    println!("if let");
    loop { let _y = mk("y"); break mk("t"); };
    println!("loop");
    'a: { let _y = mk("y"); if c { break 'a mk("t"); } mk("u") };
    println!("labeled");
    { { let _y = mk("y"); mk("t") }; println!("in"); }
    println!("nested");
    let s = mk("s");
    s;
    println!("moved local");
    let p = P { a: D("pa"), b: D("pb") };
    p.a;
    println!("moved field");
    let n = 1;
    n;
    { let _y = mk("y"); len(&mk("t")) };
    println!("end {}", n);
}
