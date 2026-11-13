// feature: cinterop.extern
// form: extern block, raw pointer types and an unsafe read, the C shim surface
unsafe extern "C" { fn abs(x: i32) -> i32; }
fn main() {
    let v = 9i32;
    let p: *const i32 = &v;
    let n = unsafe { abs(-4) + *p };
    println!("{n}");
}
