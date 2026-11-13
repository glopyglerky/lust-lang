// expect: L0110
// The later body evaluator must also use the target's mutex state width.
#[cfg(target_os = "windows")]
const WRONG: usize = 8;
#[cfg(not(target_os = "windows"))]
const WRONG: usize = 3;

fn main() {
    let _: [u8; WRONG] = [0; std::mem::size_of::<std::sync::Mutex<u8>>()];
}
