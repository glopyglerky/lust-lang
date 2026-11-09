// capability: rejected: traits, generic type aliases, and type items named like builtin types inside function bodies
// expect: L0820

fn main() {
    struct String;
    let _s = String;
}
