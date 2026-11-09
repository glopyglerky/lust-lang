// `for` loops record no pick either: the blanket
// `impl<I: Iterator> IntoIterator for I` is the only impl an
// instance can select, even when the concrete iterator type has
// an inherent `into_iter` of its own (see check/picks.rs).
struct It(u8);
struct Other(u8);

impl Iterator for It {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            self.0 -= 1;
            Some(1)
        }
    }
}

impl Iterator for Other {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            self.0 -= 1;
            Some(100)
        }
    }
}

impl It {
    fn into_iter(self) -> Other {
        Other(2)
    }
}

fn f<T: Iterator<Item = u8>>(t: T) -> u32 {
    let mut s = 0u32;
    for x in t {
        s += x as u32;
    }
    s
}

fn main() {
    println!("{}", f(It(3)));
}
