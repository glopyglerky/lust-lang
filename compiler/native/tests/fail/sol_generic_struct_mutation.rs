// expect: L0312
// Sol review probe `generic_struct_mutation`; rustc: E0506 cannot assign to `x` because it is borrowed
struct Hold<T> { r: T }
fn main() {
    let mut x = 1;
    let h = Hold { r: &x };
    x = 2;
    println!("{}", h.r);
}
