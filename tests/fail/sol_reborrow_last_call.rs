// expect: L0303
// Sol review probe `reborrow_last_call`; rustc: E0502 cannot borrow `*p` as immutable because it is also borrowed as mutable
fn read(_x: &i32, _y: &i32) {}
fn main() {
    let mut x = 0;
    let p = &mut x;
    let q = &mut *p;
    read(p, q);
}
