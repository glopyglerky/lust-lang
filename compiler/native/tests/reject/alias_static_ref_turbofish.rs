// expect: L0203
// Valid Rust. lust cannot keep an alias's `&'static` in a turbofish inside a body, so it
// rejects it as it does a written `&'static` there.
type R = &'static u32;
static N: u32 = 3;
fn main() {
    let mut v = Vec::<R>::new();
    v.push(&N);
    println!("{:?}", v);
}
