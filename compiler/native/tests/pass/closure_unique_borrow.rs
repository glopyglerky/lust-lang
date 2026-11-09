// A closure that mutates through a `&mut` binding borrows `*r`
// uniquely; `r` itself need not be `mut`.
fn main() {
    let mut v = vec![1u8];
    let r = &mut v;
    let mut push = || r.push(2);
    push();
    push();
    r.push(3);
    println!("{:?}", v);
    let mut total = 0u32;
    for i in 1..4u32 {
        let mut add = |k: u32| total += k * i;
        add(1);
        add(10);
    }
    println!("{}", total);
}
