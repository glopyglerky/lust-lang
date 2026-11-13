// expect: L0306
// rustc E0597: `mem::replace` writes `&l` into `v`, which is used after `l` ends.
fn main() {
    let k = 1u8;
    let mut v: &u8 = &k;
    {
        let l = 2u8;
        let _old = std::mem::replace(&mut v, &l);
    }
    println!("{}", v);
}
