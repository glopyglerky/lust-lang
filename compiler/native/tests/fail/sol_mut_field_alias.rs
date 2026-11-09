// expect: L0304
// Sol review probe `mut_field_alias`; rustc: E0499 cannot borrow `x` as mutable more than once at a time
struct Hold<'a> { r: &'a mut i32 }
fn main() {
    let mut x = 0;
    let h = Hold { r: &mut x };
    let other = &mut x;
    println!("{} {}", h.r, other);
}
