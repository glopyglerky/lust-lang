// feature: ext.flate2
// form: MultiGzDecoder over embedded bytes, as diagnostics_loc.rs decodes locale data
// cargo-dep: flate2 = "=1.1.10"
use std::io::Read;
static GZ: &[u8] = &[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 203, 72, 205, 201, 201, 231, 2, 0, 32, 48, 58, 54, 6, 0, 0, 0];
fn main() {
    let mut d = flate2::read::MultiGzDecoder::new(GZ);
    let mut s = String::new();
    d.read_to_string(&mut s).unwrap();
    println!("{s}");
}
