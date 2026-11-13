// expect: L0306
// Sol review probe `fnptr_elided_escape`; rustc: E0597 `x` does not live long enough
fn id(x: &i32) -> &i32 { x }
fn main() {
    let f: fn(&i32) -> &i32 = id;
    let r;
    { let x = 1; r = f(&x); }
    println!("{}", r);
}
