// expect: L0733
// rustc: lifetime may not live long enough. The collected items are the type param
// iterator's `&'a u8`, so returning one as `&'b u8` needs `'a: 'b`.
fn h<'a, 'b, I: Iterator<Item = &'a u8>>(it: I, _y: &'b u8) -> &'b u8 {
    let v: Vec<&u8> = it.collect();
    v[0]
}
fn main() {
    static X: [u8; 1] = [1];
    println!("{}", h(X.iter(), &X[0]));
}
