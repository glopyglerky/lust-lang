// feature: ext.memchr
// form: memchr, memrchr, memmem::find, and a Finder over a str needle
// cargo-dep: memchr = "=2.8.2"
use memchr::memmem;
fn main() {
    let text: &[u8] = b"let a = 1; /* x */ let b = 2;\n";
    println!("{:?} {:?}", memchr::memchr(b';', text), memchr::memrchr(b';', text));
    println!("{:?}", memmem::find(text, b"*/"));
    let f = memmem::Finder::new("let");
    println!("{:?}", f.find(&text[1..]));
}
