// A Box<[T]> mutates through DerefMut: sort, iter_mut, subslices.
fn main() {
    let mut b: Box<[i32]> = vec![3, 1, 2].into_boxed_slice();
    b.sort();
    for x in b.iter_mut() {
        *x *= 10;
    }
    b.reverse();
    let s = &mut b[1..];
    s[0] = 7;
    println!("{:?} {:?}", b, b.binary_search(&30));
}
