// Method calls through a user `Deref` nested in each other's arguments: each call
// walks its own receiver's deref chain, however deep the calls nest.
use std::cell::Cell;

pub struct Base {
    n: Cell<u32>,
}
impl Base {
    pub fn one(&self, a: u32) -> u32 {
        self.n.set(self.n.get() * 3 + 1);
        a + 1
    }
    pub fn z(&self) -> u32 {
        self.n.set(self.n.get() + 2);
        1
    }
    pub fn list(&self, xs: &[u32]) -> u32 {
        xs.iter().fold(0, |acc, x| acc * 7 + x)
    }
}
pub struct Mid(Base);
impl std::ops::Deref for Mid {
    type Target = Base;
    fn deref(&self) -> &Base {
        &self.0
    }
}
pub struct Top(Mid);
impl std::ops::Deref for Top {
    type Target = Mid;
    fn deref(&self) -> &Mid {
        &self.0
    }
}

fn main() {
    let f = &Mid(Base { n: Cell::new(0) });
    let r = f.one(f.one(f.one(f.one(f.one(f.one(f.z()))))));
    println!("{r} {}", f.n.get());
    let t = Top(Mid(Base { n: Cell::new(0) }));
    let r = t.list(&[t.z(), t.one(t.list(&[t.one(t.one(t.one(t.z()))), t.z()]))]);
    println!("{r} {}", t.n.get());
}
