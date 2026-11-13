// Closures that call methods on captured trait objects: shared, move,
// mutable, nested, generic Fn/FnMut, a supertrait upcast, and captures of
// a Vec and an Rc holding `&dyn` values.
use std::rc::Rc;
trait Read {
    fn read(&self) -> u32;
}
trait Counter {
    fn inc(&mut self) -> u32;
    fn get(&self) -> u32;
}
trait Sub: Read {
    fn extra(&self) -> u32 {
        2
    }
}
struct Number {
    n: u32,
}
impl Read for Number {
    fn read(&self) -> u32 {
        self.n
    }
}
impl Sub for Number {}
impl Counter for Number {
    fn inc(&mut self) -> u32 {
        self.n += 1;
        self.n
    }
    fn get(&self) -> u32 {
        self.n
    }
}
fn apply<F: Fn() -> u32>(f: F) -> u32 {
    f() + f()
}
fn twice<F: FnMut() -> u32>(mut f: F) -> u32 {
    f() + f()
}
fn main() {
    let n = Number { n: 7 };
    let d: &dyn Read = &n;
    let shared = || d.read() + 1;
    println!("{} {}", shared(), shared());
    let moved = move || d.read() + 2;
    println!("{} {}", moved(), d.read());
    let outer = || {
        let inner = || d.read();
        inner() + 1
    };
    println!("{} {}", outer(), apply(|| d.read()));
    let s: &dyn Sub = &n;
    let up = || {
        let b: &dyn Read = s;
        b.read() + s.extra()
    };
    println!("{}", up());
    let v: Vec<&dyn Read> = vec![d, s];
    let sum = || v[0].read() + v[1].read();
    let r = Rc::new(d);
    let rc = move || r.read() * 2;
    println!("{} {}", sum(), rc());
    let mut m = Number { n: 1 };
    let c: &mut dyn Counter = &mut m;
    let mut bump = || c.inc();
    println!("{} {}", bump(), bump());
    println!("{} {}", twice(|| c.inc()), c.get());
}
