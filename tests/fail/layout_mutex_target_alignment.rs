// expect: L0110
// A Windows signature must not use the Linux mutex alignment.
#[cfg(target_os = "windows")]
const WRONG: usize = 4;
#[cfg(not(target_os = "windows"))]
const WRONG: usize = 1;

fn bytes() -> [u8; std::mem::align_of::<std::sync::Mutex<()>>()] { [0; WRONG] }
fn main() {}
