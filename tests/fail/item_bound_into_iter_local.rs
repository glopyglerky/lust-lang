// expect: L0306
// rustc E0597: `Item = &'b u8` ties the array's items to `'b`, the Vec's
// element lifetime, so `s` must outlive the Vec.
fn add<'b, I: IntoIterator<Item = &'b u8>>(v: &mut Vec<&'b u8>, it: I) {
    for x in it {
        v.push(x);
    }
}
fn main() {
    let mut v = Vec::new();
    {
        let s = 5u8;
        add(&mut v, [&s]);
    }
    println!("{}", v.len());
}
