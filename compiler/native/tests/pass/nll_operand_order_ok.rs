// rustc accepts: a borrow whose last use is an earlier operand of the same
// statement does not conflict with later operands or the write.
fn main() {
    let mut x = 1;
    let m = &mut x;
    let y = *m + x;
    println!("{}", y);

    let mut a = 1;
    let r = &a;
    a = *r + 1;
    println!("{}", a);

    let mut t = (1, 2);
    let rt = &t;
    t = (rt.0, 3);
    println!("{:?}", t);

    let mut v = vec![1];
    let rv = &v;
    v = vec![rv.len()];
    v.push(v[0]);
    v[0] = v[1];
    println!("{:?}", v);
}
