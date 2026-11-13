// expect: L0522
// Valid Rust (an edition 2024 unsafe attribute). lust has no unsafe code.
#[unsafe(no_mangle)]
pub fn f() -> u32 {
    1
}

fn main() {
    println!("{}", f());
}
