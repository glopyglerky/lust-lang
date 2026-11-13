// expect: L0306
// rustc E0716: a later arm gets no expectation from the earlier ones,
// so the inner `if` joins its own arms and the outer LUB coerces its
// whole value after the dropped `&v[0]` temporary.
fn main() {
    let v = vec![1u8, 2];
    let k = v.len();
    let r: Option<&u8> = Some(k).map(|k| if k > 0 { &v[0] } else if k > 1 { &&v[0] } else { &v[1] });
    println!("{:?}", r);
}
