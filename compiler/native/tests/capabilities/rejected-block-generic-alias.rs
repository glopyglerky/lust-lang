// capability: rejected: traits, generic type aliases, and type items named like builtin types inside function bodies
// expect: L0820

fn main() {
    type V<T> = Vec<T>;
    let v: V<u32> = vec![1];
    println!("{}", v.len());
}
