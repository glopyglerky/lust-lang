// `[T]::swap` on arrays, slices, and generic slices (W-BENCH4
// rank 33); `Vec::swap` shares the builtin.
fn s<T>(v: &mut [T]) {
    v.swap(0, 1);
}
fn main() {
    let mut v = [1, 2, 3];
    v.swap(0, 2);
    println!("{:?}", v);
    let mut w = vec![1, 2];
    s(&mut w);
    println!("{:?}", w);
    let mut x = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    x.swap(0, 2);
    x[1..].swap(0, 1);
    println!("{:?}", x);
    let mut y = [5u8, 6, 7, 8];
    y.swap(3, 3);
    println!("{:?}", y);
}
