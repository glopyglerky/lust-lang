// expect: L0733
// rustc: lifetime may not live long enough. `Inv<'a>` is invariant in 'a
// (behind `&'a mut`), so `Inv<'b>` does not shrink to `Inv<'a>`.
struct Inv<'a> {
    r: &'a mut Vec<&'a i32>,
}

fn shrink<'a, 'b: 'a>(x: Inv<'b>) -> Inv<'a> {
    x
}

fn main() {
    let one = 1;
    let mut v = vec![&one];
    let i = shrink(Inv { r: &mut v });
    println!("{}", i.r.len());
}
