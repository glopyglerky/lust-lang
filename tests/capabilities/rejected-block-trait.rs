// capability: rejected: traits, generic type aliases, and type items named like builtin types inside function bodies
// expect: L0820

fn main() {
    trait T {
        fn f(&self) -> u32;
    }
    struct A;
    impl T for A {
        fn f(&self) -> u32 {
            1
        }
    }
    println!("{}", A.f());
}
