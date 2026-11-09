// expect: L0631
// rustc E0015: upstream's `Compression::level` is not a const fn.
const L: u32 = flate2::Compression::new(3).level();

fn main() {
    println!("{L}");
}
