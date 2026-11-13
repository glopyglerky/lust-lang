// Box<[T]> of a zero-sized type, of Vecs (clone, ==), and in a Result.
fn main() {
    let b: Box<[()]> = vec![(), (), ()].into_boxed_slice();
    println!("{} {:?}", b.len(), b);
    let v = b.into_vec();
    println!("{}", v.len());
    let e: Box<[Vec<u8>]> = vec![vec![1], vec![], vec![2, 3]].into_boxed_slice();
    let c = e.clone();
    println!("{:?} {}", c, e == c);
    let r: Result<Box<[u8]>, String> = Ok(vec![1, 2].into_boxed_slice());
    println!("{:?}", r.unwrap());
}
