// expect: L0312
// Sol review probe `call_argument_regions`; rustc: E0506 cannot assign to `x` because it is borrowed
fn use_ref(_x: &i32) {}
fn main() {
    let mut x = 0;
    let p = &x;
    x = 1;
    use_ref(p);
}
