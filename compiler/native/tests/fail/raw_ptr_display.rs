// expect: L0112
// Raw pointers have Debug but no Display.
fn main() {
    let p: *const i32 = std::ptr::null();
    println!("{p}");
}
