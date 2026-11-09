// expect: L0308
// rustc E0596: an immutable static cannot be borrowed as mutable (this
// built: every static borrow lowered to a shared reference).
fn bump(x: &mut u8) {
    *x += 1;
}
static Y: u8 = 1;
fn main() {
    bump(&mut Y);
    println!("{}", Y);
}
