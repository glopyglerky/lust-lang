// `Index::index` panics at the `[`, also when it starts the next line.
// panic: index out of bounds: the len is 3 but the index is 4
fn main() {
    let n = std::env::args().count();
    let v = vec![1, 2, 3];
    let a = v
        [n + 1];
    println!("{}", a);
    let b = v
        [n + 3];
    println!("{}", b);
}
