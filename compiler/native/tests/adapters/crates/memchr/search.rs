// memchr 2.8.2: byte and substring search on fixed inputs, including
// empty needles, needles longer than the haystack, overlapping
// patterns, and owned finders.
use memchr::memmem;

fn main() {
    let h: &[u8] = b"hello, world; hello again";
    println!("{:?} {:?}", memchr::memchr(b'l', h), memchr::memrchr(b'l', h));
    println!("{:?} {:?}", memchr::memchr(b'z', h), memchr::memrchr(b'z', h));
    let empty: &[u8] = b"";
    println!("{:?} {:?}", memchr::memchr(b'a', empty), memchr::memrchr(b'a', empty));
    println!("{:?} {:?}", memchr::memchr2(b'w', b';', h), memchr::memrchr2(b'h', b'z', h));
    println!("{:?} {:?}", memchr::memchr3(b'x', b'y', b'z', h), memchr::memrchr3(b'a', b'g', b'n', h));
    let mut it = memchr::memchr_iter(b'l', h);
    println!("{:?} {:?}", it.next(), it.next());
    let mut total = 0;
    for i in memchr::memchr2_iter(b'o', b'a', h) {
        total += i;
    }
    println!("{}", total);
    let mut seen = Vec::new();
    for i in memchr::memchr3_iter(b'h', b',', b';', h) {
        seen.push(i);
    }
    println!("{:?}", seen);
    println!("{:?} {:?} {:?}", memmem::find(h, b"hello"), memmem::rfind(h, b"hello"), memmem::find(h, b""));
    println!("{:?} {:?} {:?}", memmem::rfind(h, b""), memmem::find(b"ab", b"abc"), memmem::rfind(empty, b""));
    let aaaa: &[u8] = b"aaaa";
    let mut over = Vec::new();
    for i in memmem::find_iter(aaaa, "aa") {
        over.push(i);
    }
    let mut back = Vec::new();
    for i in memmem::rfind_iter(aaaa, "aa") {
        back.push(i);
    }
    println!("{:?} {:?}", over, back);
    let abc: &[u8] = b"abc";
    let mut e = memmem::find_iter(abc, "");
    println!("{:?} {:?} {:?} {:?} {:?}", e.next(), e.next(), e.next(), e.next(), e.next());
    let mut r = memmem::rfind_iter(abc, "");
    println!("{:?} {:?} {:?} {:?} {:?}", r.next(), r.next(), r.next(), r.next(), r.next());
    let f = memmem::Finder::new("o");
    println!("{:?} {:?}", f.find(h), f.needle());
    let mut fi = f.find_iter(h);
    println!("{:?} {:?} {:?}", fi.next(), fi.next(), fi.next());
    let rev = memmem::FinderRev::new("l");
    println!("{:?} {:?}", rev.rfind(h), rev.rfind(&h[..5]));
    let mut ri = rev.rfind_iter(h);
    println!("{:?} {:?}", ri.next(), ri.next());
    let name = String::from("ag");
    let owned = memmem::Finder::new(&name).into_owned();
    println!("{:?} {:?}", owned.find(h), owned.as_ref().needle());
    let owned_rev = memmem::FinderRev::new(&name).into_owned();
    println!("{:?}", owned_rev.rfind(h));
    let cloned = f.clone();
    println!("{:?}", cloned.find(b"xo"));
}
