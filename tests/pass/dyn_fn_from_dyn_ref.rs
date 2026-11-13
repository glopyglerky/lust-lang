// A dyn Fn / FnMut value whose concrete type is a reference to another dyn Fn object: an
// impl Fn* param forwarded as `&mut v` with v: &mut dyn FnMut (ts-rust Node::for_each_child).
fn each_dyn(n: u32, v: &mut dyn FnMut(u32) -> bool) -> bool {
    for i in 0..n { if v(i) { return true; } }
    false
}
fn each(n: u32, mut v: impl FnMut(u32) -> bool) -> bool { each_dyn(n, &mut v) }
fn walk(n: u32, v: &mut dyn FnMut(u32) -> bool) -> bool { each(n, v) }
fn walk_shared(n: u32, v: &dyn Fn(u32) -> String) -> String {
    fn fwd(n: u32, f: impl Fn(u32) -> String) -> String { let d: &dyn Fn(u32) -> String = &f; (0..n).map(|i| d(i)).collect() }
    fwd(n, v)
}
fn unit_ret(v: &mut dyn FnMut(u32)) { fn go(mut f: impl FnMut(u32)) { let d: &mut dyn FnMut(u32) = &mut f; d(5); d(6); } go(v) }
fn main() {
    let mut seen = Vec::new();
    let r = walk(6, &mut |i| { seen.push(i); i == 4 });
    println!("{r} {seen:?}");
    let k = String::from("x");
    println!("{}", walk_shared(3, &|i| format!("{k}{i}")));
    let mut sum = 0;
    unit_ret(&mut |i| sum += i);
    println!("{sum}");
}
