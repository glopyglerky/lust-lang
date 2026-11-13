// Sol review probe `call_consumed_reference`: rustc accepts.
fn read(_x: &i32) {}
fn bad<'a>(r: &'a i32) {
    let mut x = 1;
    let mut p = r;
    read(p);
    p = &x;
    x = 2;
}
fn main() { let x = 0; bad(&x); }
