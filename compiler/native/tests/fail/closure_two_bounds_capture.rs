// expect: L0306
// rustc E0597: the closure for `F` returns a borrow of `z` as `'a`, the result's lifetime.
fn two<'a, 'b>(a: &'a u8, b: &'b u8, f: impl Fn(&'a u8) -> &'a u8, g: impl Fn(&'b u8) -> &'b u8) -> &'a u8 {
    g(b);
    f(a)
}

fn main() {
    let x = 1u8;
    let r;
    {
        let z = 3u8;
        r = two(&x, &x, |_| &z, |q| q);
    }
    println!("{}", r);
}
