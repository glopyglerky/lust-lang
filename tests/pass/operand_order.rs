// Operands run left to right: a place operand is read where it is
// evaluated, before a later operand writes it.
fn f2(a: u8, b: u8) -> (u8, u8) {
    (a, b)
}
fn main() {
    let mut x = 1u8;
    let t = (x, {
        x += 1;
        x
    });
    let mut y = 1u8;
    let u = f2(y, {
        y += 1;
        y
    });
    let mut z = 1u8;
    let a = [z, {
        z += 1;
        z
    }];
    let mut w = 1u8;
    let v = vec![w, {
        w += 1;
        w
    }];
    let mut k = 1u8;
    let c = |p: u8, q: u8| p * 10 + q;
    let d = c(k, {
        k += 1;
        k
    });
    println!("{:?} {:?} {:?} {:?} {}", t, u, a, v, d);
}
