// expect: L0631
// rustc E0015: assert_ne! calls the non-const assert_failed.
const C: () = { assert_ne!(5u32, 3); };
fn main() { let _ = C; println!("ok"); }
