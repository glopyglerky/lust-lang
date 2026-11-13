// A std container over a reference to a type that is not Debug builds
// when it is never formatted (rustc checks Debug only at a `{:?}` use);
// one over a reference to a Debug type formats through it.
use std::cell::OnceCell;
use std::fmt;

struct No;
struct W(i32);
impl fmt::Debug for W {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "W<{}>", self.0)
    }
}
#[derive(Debug)]
struct G<T>(T);

static NO: No = No;
static WS: W = W(5);

#[derive(Default)]
struct S {
    f: OnceCell<&'static No>,
}

fn main() {
    let s = S::default();
    let _ = s.f.set(&NO);
    let g: OnceCell<G<&'static No>> = OnceCell::new();
    println!("{} {}", s.f.get().is_some(), g.get().is_none());
    let c: OnceCell<&'static W> = OnceCell::new();
    let _ = c.set(&WS);
    let r: &'static W = &WS;
    let d: OnceCell<G<&&W>> = OnceCell::new();
    let _ = d.set(G(&r));
    println!("{:?} {:?}", c, d);
}
