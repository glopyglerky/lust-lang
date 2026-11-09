// capability: rejected: known attributes lust does not handle yet, such as `target_feature`, `crate_name`, `no_implicit_prelude`, and unsafe ones like `#[unsafe(no_mangle)]` (named in the message)
// expect: L0522

#[unsafe(no_mangle)]
pub fn f() -> u32 {
    1
}

fn main() {
    println!("{}", f());
}
