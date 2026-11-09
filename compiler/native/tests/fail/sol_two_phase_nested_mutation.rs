// expect: L0303
// Sol review probe `two_phase_nested_mutation`; rustc: E0499 cannot borrow `v` as mutable more than once at a time
fn main() {
    let mut v = Vec::new();
    v.push({ v.push(1); 2 });
    println!("{}", v.len());
}
