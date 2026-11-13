// include_bytes! on one warm daemon: the included file changes while the
// source does not (text, then binary bytes, then a longer file), and the
// source switches to another file.
static DATA: &[u8] = include_bytes!("data.bin");
const fn sum(b: &[u8]) -> u32 {
    let mut s = 0;
    let mut i = 0;
    while i < b.len() {
        s += b[i] as u32;
        i += 1;
    }
    s
}
const SUM: u32 = sum(include_bytes!("data.bin"));
fn main() {
    let local = include_bytes!("data.bin");
    println!("{} {} {:?} {}", DATA.len(), SUM, &local[..local.len().min(4)], local.len());
}
