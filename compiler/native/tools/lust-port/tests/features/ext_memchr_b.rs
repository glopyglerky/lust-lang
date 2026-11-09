// feature: ext.memchr
// form: memchr_iter(..).count() and Finder::new(b"//"), as ts-rust scanner_util and tspath write them
// cargo-dep: memchr = "=2.8.2"
fn main() {
    let bytes: &[u8] = b"a\nb\nc";
    let lines = memchr::memchr_iter(b'\n', bytes).count() + 1;
    let slash = memchr::memmem::Finder::new(b"//");
    println!("{} {:?}", lines, slash.find(b"x//y"));
}
