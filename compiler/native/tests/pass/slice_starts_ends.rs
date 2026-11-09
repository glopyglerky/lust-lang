// `[T]::starts_with` and `ends_with` on slices, arrays, Vecs, and
// generic slices (tf-slice); the needle is any `&[T]` coercion site.
fn has<T: PartialEq>(s: &[T], n: &[T]) -> (bool, bool) {
    (s.starts_with(n), s.ends_with(n))
}
fn main() {
    let b: &[u8] = b"abc";
    println!("{} {} {}", b.starts_with(b"ab"), b.starts_with(b"ac"), b.starts_with(b"abcd"));
    println!("{} {} {}", b.ends_with(b"bc"), b.ends_with(b"ac"), b.ends_with(b"abcd"));
    println!("{} {}", b.starts_with(b""), b.ends_with(b""));
    let v = vec![1, 2, 3];
    println!("{} {}", v.starts_with(&[1, 2]), v.ends_with(&[2, 3]));
    println!("{:?}", has(&v, &[1]));
    let a = [4, 5, 6];
    println!("{} {}", a.starts_with(&[4]), a.ends_with(&[5, 6]));
    let e: &[u8] = b"";
    println!("{} {}", e.starts_with(b""), e.starts_with(b"a"));
    let w = vec!["x".to_string(), "y".to_string()];
    println!("{}", w.starts_with(&["x".to_string()]));
    let bx: Box<[u8]> = vec![9u8, 8].into_boxed_slice();
    println!("{} {}", bx.starts_with(&[9]), bx.ends_with(&[8]));
}
