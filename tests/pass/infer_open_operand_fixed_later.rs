// An operator with an open operand waits for a later use to fix it, as in
// rustc: a comparison before `push`, a closure parameter fixed by its call,
// an arithmetic result used after the call, and `sum` before `push`.
fn main() {
    let mut v = Vec::new();
    let b = v.iter().any(|x| *x > 7);
    v.push(3u8);
    println!("{b} {v:?}");
    let f = |x| x * 2;
    let y = f(3u64);
    println!("{}", y.pow(2));
    let g = |s| s == "a";
    println!("{} {}", g("a"), g("b"));
    let h = |x| x > 7;
    println!("{}", h(9u8));
    let x = "5".parse().unwrap();
    let c = x > 3;
    let z: u16 = x;
    println!("{c} {z}");
    let mut w = Vec::new();
    w.push(Default::default());
    let s: u32 = w.iter().copied().sum();
    let _keep: Vec<u32> = w;
    println!("{s}");
    let mut d = std::collections::VecDeque::from([1u8]);
    d.extend(vec![2, 3]);
    println!("{d:?}");
}
