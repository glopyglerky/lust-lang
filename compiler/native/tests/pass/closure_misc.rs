// Two-phase borrows, matches on captured places, `move` of a
// `&mut`, `mut` parameters, Copy-bounded closures, array captures.
struct P {
    a: String,
    b: String,
}
fn dup<F: Fn() -> u8 + Copy>(f: F) -> u8 {
    let g = f;
    f() + g()
}
fn main() {
    let mut v = vec![1usize];
    let c = || v.len();
    v.push(c());
    println!("{:?}", v);
    let p = P { a: String::from("a"), b: String::from("b") };
    let first = || match &p {
        P { a, .. } => a.len(),
    };
    println!("{} {} {}", first(), p.a, p.b);
    let mut x = 5;
    let r = &mut x;
    let mut inc = move || {
        *r += 1;
        *r
    };
    println!("{} {}", inc(), inc());
    println!("{}", x);
    let add = |mut x: u8, y: u8| {
        x += y;
        x
    };
    println!("{}", add(1, 2));
    let k = 4u8;
    println!("{}", dup(|| k));
    let mut b = [0u8; 3];
    let mut set = |i: usize, val: u8| b[i] = val;
    set(0, 7);
    set(2, 9);
    println!("{:?}", b);
    let t = (1u8, String::from("x"));
    let f = || t.0 + 1;
    let s = t.1;
    println!("{} {}", f(), s);
}
