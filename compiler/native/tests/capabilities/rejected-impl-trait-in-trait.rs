// capability: rejected: `impl Trait` inside another type (`Vec<impl Fn()>`), in trait method return types, and outside fn signatures
// expect: L0602

trait T {
    fn it(&self) -> impl Iterator<Item = u32>;
}

struct A;

impl T for A {
    fn it(&self) -> impl Iterator<Item = u32> {
        0..3
    }
}

fn main() {
    println!("{}", A.it().sum::<u32>());
}
