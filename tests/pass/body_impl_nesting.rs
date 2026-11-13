// Trait impls in a nested block, a closure body, a const initializer, and a method body.
trait N { fn n(&self) -> u32; }
struct A; struct B; struct C; struct D;
fn show(x: &dyn N) -> u32 { x.n() }
struct H;
impl H {
    fn m(&self) -> u32 {
        impl N for D { fn n(&self) -> u32 { 4 } }
        show(&D)
    }
}
fn main() {
    {
        impl N for A { fn n(&self) -> u32 { 1 } }
    }
    let c = || {
        impl N for B { fn n(&self) -> u32 { 2 } }
        show(&B)
    };
    const K: u32 = {
        impl N for C { fn n(&self) -> u32 { 3 } }
        7
    };
    println!("{} {} {} {} {}", show(&A), c(), show(&C), H.m(), K);
}
