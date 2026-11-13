mod sub;
#[path = "sub/defs.rs"]
mod defs;

include!(concat!(env!("OUT_DIR"), "/gen.rs"));
include!("sub/part.rs");

fn main() {
    println!("lib={}", sub::lib_text().trim());
    println!("bytes={}", sub::lib_bytes_len());
    println!("call={}", load_calldata!());
    println!("inner={}", inner_text());
    println!("gen={}", generated());
    println!("outdata={}", include_str!(concat!(env!("OUT_DIR"), "/data.txt")));
    println!("greeting={}", env!("GREETING"));
    println!("has_gen={}", cfg!(has_gen));
    println!("pkg={} {} {}.{}.{} pre={}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_VERSION_MAJOR"), env!("CARGO_PKG_VERSION_MINOR"), env!("CARGO_PKG_VERSION_PATCH"), env!("CARGO_PKG_VERSION_PRE"));
    println!("authors={}", env!("CARGO_PKG_AUTHORS"));
    println!("desc={}", env!("CARGO_PKG_DESCRIPTION"));
    println!("crate={} bin={}", env!("CARGO_CRATE_NAME"), env!("CARGO_BIN_NAME"));
    println!("manifest_ok={}", env!("CARGO_MANIFEST_PATH") == concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
    println!("dep={} {} {}", genlib::dep_pkg(), genlib::deep::dep_out(), genlib::deep::dep_deep());
}
