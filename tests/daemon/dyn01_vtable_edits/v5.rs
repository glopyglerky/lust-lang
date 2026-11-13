// Dyn edits on one warm daemon: method bodies, the concrete type's layout
// (the vtable size word), a new trait method (vtable slots), the `'static`
// flag of `impl dyn Tr`, and dyn compatibility.
trait Tr {
    fn f(&self) -> u32;
    fn g(&self) -> u32 {
        self.f() + 1000
    }
}
impl dyn Tr {
    fn twice(&self) -> u32 {
        self.f() * 2
    }
}
struct A {
    pad: [u64; 3],
    x: u32,
}
struct B(u8);
impl Tr for A {
    fn f(&self) -> u32 {
        self.x + 10 + self.pad[0] as u32
    }
}
impl Tr for B {
    fn f(&self) -> u32 {
        self.0 as u32 + 100
    }
}
fn show(d: &dyn Tr) -> u32 {
    d.twice() + d.f() + d.g()
}
fn main() {
    let a = A { pad: [7; 3], x: 3 };
    let b = B(4);
    let ds: [&dyn Tr; 2] = [&a, &b];
    for d in ds {
        println!("{} {}", show(d), std::mem::size_of_val(d));
    }
}
