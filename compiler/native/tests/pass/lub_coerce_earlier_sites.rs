// rustc's LUB coercion coerces the earlier sites to a later type when
// only that direction works: `[&&d, &d]` derefs the first element, and
// an array reference arm or branch before a slice one unsizes.
#[derive(Debug)]
struct D(u8);
fn pick(n: u8, v: &[u8]) -> usize {
    let three = [1u8, 2, 3];
    let s = match n {
        0 => &three,
        _ => v,
    };
    let t = if n == 0 { &three } else { v };
    let u = match n {
        0 => v,
        _ => &three,
    };
    s.len() * 100 + t.len() * 10 + u.len()
}
fn main() {
    println!("{} {}", pick(0, &[1, 2]), pick(2, &[1, 2, 3, 4]));
    let a = D(1);
    let b = D(2);
    let c = D(3);
    let ra = &a;
    let rc = &c;
    let arr = [&ra, &rc, &b];
    println!("{:?}", arr);
    let arr2 = [&b, &ra];
    println!("{:?}", arr2);
    let x = match a.0 {
        0 => &ra,
        _ => &b,
    };
    println!("{:?}", x);
}
