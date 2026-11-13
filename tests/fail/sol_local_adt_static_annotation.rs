// expect: L0306
// Sol review probe `local_adt_static_annotation`; rustc: E0597 `x` does not live long enough
struct Hold<'a> { r: &'a i32 }
fn main() {
    let x = 1;
    let h: Hold<'static> = Hold { r: &x };
    println!("{}", h.r);
}
