// memchr 2.8.2 with array needles and haystacks, as ts-rust writes
// `memmem::Finder::new(b"//")` (`[u8; N]: AsRef<[u8]>`).
use memchr::memmem;

fn main() {
    let f = memmem::Finder::new(b"//");
    println!("{:?}", f.find(b"a//b"));
    let r = memmem::FinderRev::new(b"/.");
    println!("{:?}", r.rfind(b"a/./b/."));
    println!("{:?}", memmem::find(b"x*/y", b"*/"));
}
