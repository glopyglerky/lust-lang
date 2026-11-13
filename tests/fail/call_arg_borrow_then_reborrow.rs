// expect: L0303
// rustc E0502: `&c.k` is still live when the later argument reborrows `c`.
struct S { k: u32 }
fn h(a: &u32, b: &mut S) -> u32 { b.k + *a }
fn main() {
    let mut s = S { k: 1 };
    let c = &mut s;
    println!("{}", h(&c.k, c));
}
