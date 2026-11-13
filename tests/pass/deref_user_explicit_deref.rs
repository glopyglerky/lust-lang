// An explicit `*w` through a user `Deref` impl, next to an inherent
// method of the same name on the outer type (it wins without the `*`).
use std::ops::Deref;
struct N;
impl N {
    fn name(&self) -> &'static str {
        "inner"
    }
}
struct W {
    n: N,
}
impl W {
    fn name(&self) -> &'static str {
        "outer"
    }
}
impl Deref for W {
    type Target = N;
    fn deref(&self) -> &N {
        &self.n
    }
}
fn main() {
    let w = W { n: N };
    println!("{} {}", w.name(), (*w).name());
}
