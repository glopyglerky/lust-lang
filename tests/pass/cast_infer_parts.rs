fn f(v: u8) -> u8 {
    v + 1
}
fn g(v: u16) -> u32 {
    v as u32 + 2
}
fn main() {
    let a: fn(u8) -> u8 = f as fn(_) -> u8;
    let b: fn(u8) -> u8 = f as fn(u8) -> _;
    let c: fn(u16) -> u32 = g as fn(_) -> _;
    println!("{} {} {}", a(3), b(4), c(5));
    let x = (1u8, 2u32);
    let p = &x.0 as *const _ as usize;
    println!("{}", p != 0);
    let q: *const u32 = &x.1 as *const _;
    println!("{}", !q.is_null());
    let y = 7u16;
    let r = &y as *const _ as *const u16;
    println!("{}", !r.is_null());
    let h = f as fn(_) -> u8;
    println!("{}", h(9u8));
    let z: u16 = 3u8 as _;
    println!("{}", z);
}
