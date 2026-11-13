// expect: L0110
struct Buffer { bytes: [u8; core::mem::size_of::<([u8; 4611686018427387904], [u8; 4611686018427387904])>()] }
fn main() {}
