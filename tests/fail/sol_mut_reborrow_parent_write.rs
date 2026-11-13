// expect: L0312
// Sol review probe `mut_reborrow_parent_write`; rustc: E0506 cannot assign to `*p` because it is borrowed
fn main() {
    let mut x = 0;
    let p = &mut x;
    let q = &mut *p;
    *p = 1;
    println!("{}", q);
}
