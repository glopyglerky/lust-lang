// expect: L0735
// rustc: implementation of `Fn` is not general enough: a fn pointer that
// needs `&'static u8` does not become a `dyn Fn(&u8)`.
fn id(x: &'static u8) -> u8 { *x }
fn main() { let p: fn(&'static u8) -> u8 = id; let q: Box<dyn Fn(&u8) -> u8> = Box::new(p); let a = 1; println!("{}", q(&a)); }
