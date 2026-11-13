// `Index::index` (a Vec) panics at the brackets; a built-in array
// index panics at the whole expression, also inside a call argument.
// panic: index out of bounds: the len is 3 but the index is 4
fn twice(x: i32) -> i32 {
    x * 2
}

fn main() {
    let n = std::env::args().count();
    let a = [1, 2, 3];
    let v = vec![10, 20, 30];
    println!("{}", twice(a[n]) + twice((v)[n + 1]));
    let w = vec![v.clone(), v.clone()];
    println!("{}", w[n][n]);
    println!("{}", twice(w[0][n + 3]));
}
