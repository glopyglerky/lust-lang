// Sol review probe `reborrow_last_assignment`: rustc accepts.
fn main() {
    let mut x = 1;
    let p = &mut x;
    let q = &mut *p;
    *p = *q + 1;
    println!("{}", x);
}
