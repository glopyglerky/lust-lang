// `Index::index` panics at the `[`, whatever spacing follows it.
// panic: start byte index 9 is out of bounds for string of length 3
fn main() {
    let n = std::env::args().count();
    let v = vec![1, 2, 3];
    println!("{:?}", &v[ 0..n ]);
    let s = String::from("abc");
    println!("{}", &s[
        n + 8..
    ]);
}
