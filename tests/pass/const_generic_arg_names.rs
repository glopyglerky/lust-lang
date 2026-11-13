// Const generic arguments written as bare const names: a
// single-segment path that names no type reads the const item
// or the enclosing const parameter; multi-segment const paths
// take braces, as in rustc.
const N: usize = 2;

struct Arr<const M: usize> {
    data: [i32; M],
}
struct W<const M: usize> {
    a: Arr<M>,
}
struct S;
impl S {
    const K: usize = 3;
}

fn len_of<const M: usize>(a: [i32; M]) -> usize {
    a.len() + a[0] as usize
}
fn in_generic<T>(t: T) -> i32 {
    let _ = &t;
    let a: Arr<N> = Arr { data: [5, 6] };
    a.data[1]
}

fn main() {
    let w: W<N> = W { a: Arr { data: [7, 8] } };
    println!("{}", w.a.data[1]);
    println!("{}", in_generic(0));
    println!("{}", len_of::<N>([10, 20]));
    println!("{}", len_of::<{ S::K }>([10, 20, 30]));
    let b: Arr<{ S::K }> = Arr { data: [1, 2, 9] };
    println!("{}", b.data[2]);
}
