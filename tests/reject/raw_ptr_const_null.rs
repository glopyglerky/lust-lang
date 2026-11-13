// expect: L0630
// A null pointer const: const eval has no raw pointer values yet.
const P: *const i32 = std::ptr::null();
fn main() {
    println!("{}", P.is_null());
}
