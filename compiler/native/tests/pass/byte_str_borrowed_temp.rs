// A byte string literal is static data (`&'static [u8; N]`): a
// borrow of it stored in a local struct stays usable after the
// statement, as `memmem::Finder::new(b"//")` writes it. The
// literal's backing local is a const blob, so its storage ending
// is not a drop of borrowed data.
struct Finder<'n> {
    needle: &'n [u8],
}
impl<'n> Finder<'n> {
    fn new<B: ?Sized + AsRef<[u8]>>(needle: &'n B) -> Finder<'n> {
        Finder { needle: needle.as_ref() }
    }
    fn find(&self, haystack: &[u8]) -> Option<usize> {
        let n = self.needle.len();
        if haystack.len() < n {
            return None;
        }
        let mut i = 0;
        while i + n <= haystack.len() {
            if &haystack[i..i + n] == self.needle {
                return Some(i);
            }
            i += 1;
        }
        None
    }
}

fn main() {
    let f = Finder::new(b"//");
    println!("{:?}", f.find(b"a//b"));
    let s: &[u8] = b"ab";
    println!("{:?}", s);
}
