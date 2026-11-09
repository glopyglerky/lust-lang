// expect: L0303
// Sol review probe `reborrow_last_binary`; rustc: E0503 cannot use `*p` because it was mutably borrowed
fn main() {
    let mut x = 1;
    let p = &mut x;
    let q = &mut *p;
    let z = *p + *q;
    println!("{}", z);
}
