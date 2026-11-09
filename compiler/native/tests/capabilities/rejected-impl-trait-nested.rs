// capability: rejected: `impl Trait` inside another type (`Vec<impl Fn()>`), in trait method return types, and outside fn signatures
// expect: L0602

fn adders() -> Vec<impl Fn(u32) -> u32> {
    vec![|x| x + 1]
}

fn main() {
    println!("{}", adders()[0](1));
}
