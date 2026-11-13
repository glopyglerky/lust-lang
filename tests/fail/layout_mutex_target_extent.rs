// expect: L0110
// A Windows signature must not use the Linux mutex extent.
#[cfg(target_os = "windows")]
const WRONG: usize = 8;
#[cfg(not(target_os = "windows"))]
const WRONG: usize = 2;

fn bytes() -> [u8; std::mem::size_of::<std::sync::Mutex<()>>()] { [0; WRONG] }
fn main() {}
