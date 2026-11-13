// expect: L0733
fn f(slot: &mut &i32, a: &i32) {
    let mut r = a;
    r = *slot;
    *slot = r;
}

fn main() {
    let a = 1;
    let b = 2;
    let mut s = &b;
    f(&mut s, &a);
    println!("{}", s);
}
