// capability: rejected: `dyn` coercions in const and static initializers
// expect: L0702

trait T {
    fn f(&self) -> u32;
}

struct A;

impl T for A {
    fn f(&self) -> u32 {
        1
    }
}

const X: &dyn T = &A;

fn main() {
    println!("{}", X.f());
}
