// rustc promotes a numeric cast of a promotable operand and a fn item
// coerced to a fn pointer (a pointer to integer cast is never promoted).
fn f0() {}
const C: i32 = 300;
fn main() {
    let r: &'static Option<fn()> = &(Some(f0));
    let s: &'static fn() = &(f0 as fn());
    let t: &'static u8 = &(C as u8);
    println!("{} {}", r.is_some(), t);
    s();
    println!("{}", std::ptr::addr_eq(&Some(f0 as fn()), &Some(f0 as fn())));
    println!("{} {}", std::ptr::addr_eq(&(C as u8), &(C as u8)), std::ptr::addr_eq(&((1 + 2) as u16), &3u16));
    println!("{}", std::ptr::addr_eq(&(-1i8 as u32), &u32::MAX));
    let x = 5;
    println!("{}", std::ptr::addr_eq(&(x as u8), &(x as u8)));
}
