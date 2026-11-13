// An associated fn (no `self`) is no method candidate: `w.get()` finds the
// deref target's method, or a trait method, past an associated fn of the
// same name, as in rustc.
use std::ops::Deref;

struct Inner(u8);
impl Inner {
    fn get(&self) -> u8 {
        self.0 + 1
    }
}

struct W(Inner);
impl W {
    fn get(this: &W) -> u8 {
        this.0.0 + 100
    }
}
impl Deref for W {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}

trait Name {
    fn name(&self) -> &'static str;
}
struct V;
impl V {
    fn name() -> &'static str {
        "assoc"
    }
}
impl Name for V {
    fn name(&self) -> &'static str {
        "trait"
    }
}

struct G<T>(Vec<T>);
impl<T> G<T> {
    fn first(this: &G<T>) -> Option<&T> {
        this.0.first()
    }
}
impl<T> Deref for G<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.0
    }
}

fn main() {
    let w = W(Inner(5));
    println!("{} {}", w.get(), W::get(&w));
    println!("{} {}", V.name(), V::name());
    let g = G(vec![7u8, 8]);
    println!("{:?} {:?}", g.first(), G::first(&g));
}
