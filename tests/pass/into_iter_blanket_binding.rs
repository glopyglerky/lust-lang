// `I: IntoIterator<Item = u32>` holds for a user iterator through
// core's blanket `impl<I: Iterator> IntoIterator for I`: the binding
// compares with the blanket's `Item = <I as Iterator>::Item`,
// normalized through the user impl (or the param env).
struct C {
    n: u32,
}
impl Iterator for C {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.n < 3 {
            self.n += 1;
            Some(self.n)
        } else {
            None
        }
    }
}
struct D {
    n: u8,
}
impl Iterator for D {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.n < 2 {
            self.n += 1;
            Some(self.n)
        } else {
            None
        }
    }
}
fn g<I: IntoIterator<Item = u32>>(i: I) -> u32 {
    let mut s = 0;
    for x in i {
        s += x;
    }
    s
}
fn h<J: Iterator<Item = u32>>(j: J) -> u32 {
    g(j) + 100
}
fn k<I: IntoIterator<Item = u8>>(i: I) -> u32 {
    let mut s = 0u32;
    for x in i {
        s += x as u32;
    }
    s
}
fn main() {
    println!("{} {} {}", g(C { n: 0 }), h(C { n: 1 }), k(D { n: 0 }));
}
