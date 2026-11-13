#[cfg(target_os = "linux")]
const fn k() -> u32 { 3 }
#[cfg(not(target_os = "linux"))]
const fn k() -> u32 { 4 }
const X: u32 = k();
fn main() { println!("{}", X); }
