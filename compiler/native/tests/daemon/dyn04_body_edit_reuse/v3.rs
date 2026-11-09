// Body-only edits on a warm daemon reuse the stored discovery of the
// bodies they do not touch (driver/gen_reuse.rs). `main` makes the
// vtables of a generic impl (only coercion sites request those); its
// replayed discovery must request them again, or the edited build has
// no vtable to point the fat pointers at.
trait T {
    fn read(&self) -> u32;
    fn twice(&self) -> u32 {
        self.read() * 2
    }
}
struct S {
    n: u32,
}
struct U {
    n: u32,
}
impl T for S {
    fn read(&self) -> u32 {
        self.n
    }
}
impl T for U {
    fn read(&self) -> u32 {
        self.n + 200
    }
}
struct W<X> {
    x: X,
}
impl<X: Copy> T for W<X> {
    fn read(&self) -> u32 {
        50
    }
}
fn helper() -> u32 {
    2
}
fn show(d: &dyn T) -> u32 {
    d.read() + d.twice()
}
fn main() {
    let s = S { n: 3 };
    let u = U { n: 4 };
    let w = W { x: 7u8 };
    let ds: [&dyn T; 3] = [&s, &u, &w];
    println!("{} {} {} {}", show(ds[0]), show(ds[1]), show(ds[2]), helper());
}
