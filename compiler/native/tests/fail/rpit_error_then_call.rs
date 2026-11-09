// expect: L0110
// rustc E0308: a check error elsewhere in the body must be reported by build, not lost to a crash in instance collection (hunt2 rpit_error_ice).
fn it() -> impl Iterator<Item = u32> { 0..3u32 }
fn main() {
    let s: u32 = 5u8;
    for x in it() { println!("{} {}", x, s); }
}
