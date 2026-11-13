// capability: rejected: generic parameter defaults that name a lifetime
// expect: L0614

struct S<'a, T = &'a u32> {
    x: T,
    r: &'a u8,
}

fn main() {
    let (a, b) = (1, 2u8);
    let s: S = S { x: &a, r: &b };
    println!("{} {}", s.x, s.r);
}
