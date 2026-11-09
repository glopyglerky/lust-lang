// feature: unsafe.blocks
// form: unsafe block and unsafe fn without any C interop
unsafe fn read(p: *const i32) -> i32 { unsafe { *p } }
fn main() { let v = 4; println!("{}", unsafe { read(&v) }); }
