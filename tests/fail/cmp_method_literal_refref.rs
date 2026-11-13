// expect: L0612
// rustc E0277: an integer literal's type has an impl per integer type,
// so `x.partial_cmp(&x)` with `x: &{integer}` compares with `&{integer}`.
fn main() {
    let a = 5;
    let x = &a;
    println!("{:?}", x.partial_cmp(&x));
}
