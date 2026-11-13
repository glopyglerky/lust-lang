// A user trait method of `str` called on a Cow<str> through its
// Deref view, and an inherent method and field of a sized `B`.
use std::borrow::Cow;

trait Twice {
    fn twice(&self) -> usize;
}
impl Twice for str {
    fn twice(&self) -> usize {
        self.len() * 2
    }
}

#[derive(Clone)]
struct D {
    n: i32,
    s: String,
}
impl D {
    fn total(&self) -> i32 {
        self.n + self.s.len() as i32
    }
}

fn main() {
    let c: Cow<str> = Cow::Borrowed("ab");
    println!("{}", c.twice());
    let d = D { n: 10, s: "xyz".to_string() };
    let cd: Cow<D> = Cow::Borrowed(&d);
    println!("{} {} {}", cd.total(), cd.n, cd.s);
    let owned: Cow<D> = Cow::Owned(d.clone());
    println!("{} {}", owned.total(), owned.s.len());
    let v: Cow<[u8]> = Cow::Owned(vec![3, 1, 2]);
    println!("{} {:?} {}", v[1], &v[1..], v.iter().sum::<u8>());
    let w: Cow<str> = Cow::Borrowed("hello");
    println!("{}", &w[1..3]);
}
