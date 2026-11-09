// capability: rejected: a non-pub member of an impl inside a function body when a block `use` is in scope or the self type does not resolve
// expect: L0820

fn main() {
    use std::fmt::Write;
    struct S;
    impl S {
        fn f(&self) -> u32 {
            1
        }
    }
    let mut s = String::new();
    write!(s, "{}", S.f()).unwrap();
    println!("{s}");
}
