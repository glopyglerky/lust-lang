// expect: L0306
// Sol review probe `nested_field_escape`; rustc: E0597 `x` does not live long enough
struct Inner<'a> { r: &'a i32 }
struct Outer<'a> { inner: Inner<'a> }
fn main() {
    let value;
    { let x = 1; value = Outer { inner: Inner { r: &x } }; }
    println!("{}", value.inner.r);
}
