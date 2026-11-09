// feature: unsupported.other
// form: a union type, the valid-Rust member of this bucket that a probe can run
union Bits { i: i32, u: u32 }
fn main() { let b = Bits { i: -1 }; println!("{}", unsafe { b.u } % 10); }
