// expect: L0306
// rustc E0597: `two` returns `'b`, the borrow of `y`, which ends before `r` is used.
fn two<'a, 'b, F: Fn(&'a u8) -> &'a u8, G: Fn(&'b u8) -> &'b u8>(a: &'a u8, b: &'b u8, f: F, g: G) -> &'b u8 {
    f(a);
    g(b)
}

fn main() {
    let x = 1u8;
    let r;
    {
        let y = 2u8;
        r = two(&x, &y, |p| p, |q| q);
    }
    println!("{}", r);
}
