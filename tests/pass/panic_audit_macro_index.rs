// `Index::index` inside a macro body panics at the macro call.
// panic: index out of bounds: the len is 3 but the index is 3
macro_rules! at {
    ($v:expr, $i:expr) => {
        $v[$i + 3]
    };
}
fn main() {
    let n = std::env::args().count();
    let v = vec![1u8, 2, 3];
    println!("{}", at!(v, n - 1));
    let a = at!(v, n);
    println!("{}", a);
}
