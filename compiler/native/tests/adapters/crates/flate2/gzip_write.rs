// flate2 1.1.10 write side, the ts-rust pprof shape (pprof.rs:576):
// GzEncoder::new(w, Compression::fast()), write_all, finish. The
// adapter compresses with a copy of miniz_oxide 0.9.1's deflate, so
// the gzip bytes must equal the real crate's, header to trailer.
use std::io::Write;

fn main() {
    let mut out: Vec<u8> = Vec::new();
    {
        let mut zw = flate2::write::GzEncoder::new(&mut out, flate2::Compression::fast());
        zw.write_all(b"profile bytes profile bytes profile bytes").unwrap();
        zw.finish().unwrap();
    }
    println!("{} {:?}", out.len(), out);
}
