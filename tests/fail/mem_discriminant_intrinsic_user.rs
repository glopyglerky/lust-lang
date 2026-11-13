// expect: L0642
// rustc E0658 (core_intrinsics is unstable): only lust's shipped std may
// call the discriminant reads behind mem::discriminant.
enum E {
    A,
}
fn main() {
    let e = E::A;
    println!("{}", core::intrinsics::discriminant_value(&e));
}
