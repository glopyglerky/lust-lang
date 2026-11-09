fn inc(x: &mut i32) {
    *x += 1;
}
fn dbl(x: &mut i32) {
    *x *= 5;
}
static OPS: [(&str, fn(&mut i32)); 2] = [("inc", inc), ("dbl", dbl)];
fn main() {
    let mut v = 3;
    for (n, f) in &OPS {
        f(&mut v);
        println!("{} {}", n, v);
    }
}
