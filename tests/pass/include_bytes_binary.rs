// include_bytes! of binary data: `&'static [u8; N]` with the exact bytes,
// in a static, a const fn evaluated at compile time, and a local (W-CHECK-G3).
static DATA: &[u8] = include_bytes!("include_bytes_data.bin");
const N: usize = include_bytes!("include_bytes_data.bin").len();
const fn fnv(b: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    let mut i = 0;
    while i < b.len() {
        h = (h ^ b[i] as u64).wrapping_mul(0x100000001b3);
        i += 1;
    }
    h
}
const H: u64 = fnv(include_bytes!("include_bytes_data.bin"));
fn main() {
    let arr: &[u8; 309] = include_bytes!("include_bytes_data.bin");
    println!("{} {} {}", DATA.len(), N, H);
    println!("{:?}", &arr[..12]);
    println!("{} {}", fnv(DATA) == H, DATA.iter().map(|&b| b as u32).sum::<u32>());
}
