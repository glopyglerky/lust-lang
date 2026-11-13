// expect: L0617
// rustc E0478: In<'a> needs 'a: 'static, which S does not declare.
struct In<'a: 'static>(&'a u8);
struct S<'a>(In<'a>);
fn main() {
    let _s = S(In(&3));
}
