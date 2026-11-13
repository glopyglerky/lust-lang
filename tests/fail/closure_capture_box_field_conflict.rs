// expect: L0303
// A closure that mutates `(*b).x` conflicts with a read of `b.x`.
struct S { x: u32, y: u32 }
fn main() { let mut b = Box::new(S { x: 1, y: 2 }); let mut f = || b.x += 1; let r = b.x; f(); println!("{} {}", r, b.y); }
