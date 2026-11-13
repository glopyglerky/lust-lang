// expect: L0104
// rustc E0277: Vec<T>: From<&[T]> needs T: Clone.
struct N;

fn main() {
    let v: Vec<N> = (&[N][..]).into();
    println!("{}", v.len());
}
