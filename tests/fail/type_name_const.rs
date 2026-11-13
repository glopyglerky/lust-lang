// expect: L0630
// `type_name` is not a const fn in rustc either.
const S: &str = std::any::type_name::<u8>();
fn main() {
    println!("{S}");
}
