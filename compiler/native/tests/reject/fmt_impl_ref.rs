// expect: L0641
// Valid Rust: `&My` is local through the fundamental `&`, so a Debug impl
// for it is legal; lust's fmt glue would format the pointee instead.
use std::fmt;

struct My(u8);

impl fmt::Debug for &My {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ref {}", self.0)
    }
}

fn main() {
    let m = My(3);
    println!("{:?}", &m);
    let v = vec![&m, &m];
    println!("{:?}", v);
}
