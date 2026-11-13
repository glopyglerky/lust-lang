// expect: L0303
// Sol review probe `reborrow_last_print`; rustc: E0502 cannot borrow `p` as immutable because it is also borrowed as mutable
fn main() {
    let mut x = 0;
    let p = &mut x;
    let q = &mut *p;
    println!("{} {}", p, q);
}
