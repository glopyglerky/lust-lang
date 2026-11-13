// expect: L0104
// rustc E0599: `type_id` on a concrete type needs `Any` in scope (W-CHECK-G3).
fn main() { let x = 1u8; println!("{}", x.type_id() == std::any::TypeId::of::<u8>()); }
