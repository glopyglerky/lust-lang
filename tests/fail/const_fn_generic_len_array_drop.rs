// expect: L0631
// rustc E0493: `[T; N]` may hold elements with a destructor when N is not 0.
const fn f<T, const N: usize>(a: [T; N]) -> usize {
    N
}
fn main() {
    println!("{}", f([String::new()]));
}
