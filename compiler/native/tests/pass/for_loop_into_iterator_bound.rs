// A `for` loop over a bare `IntoIterator` bound: the env proves
// `into_iter`, and `next` answers from the same bound (the
// `IntoIter: Iterator` leg of the trait definition).
struct It(u8);
struct Src;

impl Iterator for It {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            self.0 -= 1;
            Some(2)
        }
    }
}

impl IntoIterator for Src {
    type Item = u8;
    type IntoIter = It;
    fn into_iter(self) -> It {
        It(3)
    }
}

fn f<T: IntoIterator<Item = u8>>(t: T) -> u32 {
    let mut s = 0u32;
    for x in t {
        s += x as u32;
    }
    s
}

fn main() {
    println!("{}", f(Src));
}
