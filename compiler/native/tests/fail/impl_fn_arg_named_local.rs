// expect: L0306
// rustc E0597: the result of `impl Fn(&'a [u8]) -> &'a u8` borrows what `'a` borrows.
fn keep<'a>(v: &'a [u8], f: impl Fn(&'a [u8]) -> &'a u8) -> &'a u8 {
    f(v)
}

fn main() {
    let r;
    {
        let w = vec![1u8, 2];
        r = keep(&w, |s| &s[1]);
    }
    println!("{}", r);
}
