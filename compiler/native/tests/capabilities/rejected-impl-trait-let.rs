// capability: rejected: `impl Trait` inside another type (`Vec<impl Fn()>`), in trait method return types, and outside fn signatures
// expect: L0602

fn main() {
    let x: impl Fn() = || {};
    x();
}
