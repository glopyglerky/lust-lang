// expect: L0110
// Valid Rust: early signature layout queries still lack user Adt shapes.
// Ordinary expression layout queries cover this uninhabited payload.
enum Empty {}

fn bytes() -> [u8; std::mem::size_of::<std::sync::OnceLock<Empty>>()] {
    loop {}
}

fn main() {}
