// expect: L0306
// rustc E0597: the rest binding borrows `v`, which dies at the end of the
// block while `r` still holds the rest.
fn main() {
    let r;
    {
        let v = vec![1u8, 2, 3];
        let mut rest = &v[..];
        while let [_, tail @ ..] = rest {
            rest = tail;
        }
        r = rest;
    }
    println!("{:?}", r);
}
