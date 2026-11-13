// expect: L0306
// Sol review probe `struct_shared_escape`; rustc: E0597 `x` does not live long enough
struct Hold<'a> { r: &'a i32 }
fn main() {
    let h;
    { let x = 1; h = Hold { r: &x }; }
    println!("{}", h.r);
}
