// expect: L0546
// rustc denies `enum_intrinsics_non_enums`: the discriminant of a struct
// is unspecified.
struct S(u8);
fn main() {
    let d = std::mem::discriminant(&S(1));
    println!("{d:?}");
}
