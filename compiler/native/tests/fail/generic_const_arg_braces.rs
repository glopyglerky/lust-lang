// expect: L0110
struct S;
impl S {
    const N: usize = 2;
}
struct Arr<const M: usize> {
    data: [i32; M],
}
fn main() {
    let a: Arr<S::N> = Arr { data: [5, 6] };
    println!("{}", a.data[1]);
}
