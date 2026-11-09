// expect: L0306
// rustc E0597: `retain_mut` hands out `&mut T`, so the predicate stores `&loc` in the Vec.
static A: u8 = 1;
fn main() {
    let mut v: Vec<&u8> = vec![&A];
    {
        let loc = 9u8;
        v.retain_mut(|x| {
            *x = &loc;
            true
        });
    }
    println!("{}", v.len());
}
