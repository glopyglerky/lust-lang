// Comparisons across sequence forms (tf-slice): core's
// asymmetric slice/array/Vec `PartialEq` impls and the slice
// `PartialOrd` coercions, with values (the accept matrix is the
// census probe in the package report).
fn main() {
    let b: &[u8] = b"ab";
    println!("{} {}", b == b"ab", b != b"ac");
    let v = vec![1, 2];
    let a = [1, 2];
    let s: &[i32] = &v;
    println!("{} {} {}", v == a, v == &a, v == s);
    println!("{} {} {}", s == v, s == a, s == &a);
    println!("{} {}", a == s, a == *s);
    println!("{} {}", *s == a, *s == v);
    println!("{} {}", &v == &a, &a == s);
    let w = vec![1, 3];
    println!("{} {} {}", v != w, s != &w[..], a != w[..]);
    println!("{} {} {}", s < &a, s <= &a, s < &w[..]);
    println!("{} {}", v < w, a < [1, 3]);
    println!("{}", b < b"ac");
    let words = vec!["a".to_string(), "b".to_string()];
    println!("{}", words == ["a".to_string(), "b".to_string()]);
    println!("{}", words[..] == ["a".to_string(), "b".to_string()][..]);
    let g: &[u8] = b"0";
    println!("{}", g != b"1" && g == b"0");
}
