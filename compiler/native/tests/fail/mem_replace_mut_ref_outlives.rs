// expect: L0306
// rustc E0597: after mem::replace, `r` holds `&mut y`, and `y` dies before `r`'s last use.
fn main() {
    let mut x = 1;
    let mut r: &mut i32 = &mut x;
    {
        let mut y = 2;
        let old = std::mem::replace(&mut r, &mut y);
        *old = 5;
    }
    *r = 6;
}
