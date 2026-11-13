// `[T]::fill`, `copy_from_slice`, and `copy_within` (tf-slice):
// value, drop, and overlap behavior. String elements show the
// clone counts of `fill` (clones but the last slot, drops in order).
fn main() {
    let mut v = vec![1, 2, 3];
    v.fill(9);
    println!("{:?}", v);
    let mut a = [0u8; 4];
    a.fill(7);
    println!("{:?}", a);
    let mut e: Vec<i32> = vec![];
    e.fill(5);
    println!("{:?}", e);
    let mut s = vec!["a".to_string(), "bb".to_string(), "ccc".to_string()];
    s.fill("z".to_string());
    println!("{:?}", s);
    s[1..].fill("q".to_string());
    println!("{:?}", s);

    let mut d = [0u8; 4];
    d.copy_from_slice(b"abcd");
    println!("{:?}", d);
    let mut w = vec![0; 3];
    w[..2].copy_from_slice(&[5, 6]);
    println!("{:?}", w);
    let src = vec![7, 8, 9];
    w.copy_from_slice(&src);
    println!("{:?}", w);

    let mut c = vec![0, 1, 2, 3, 4, 5];
    c.copy_within(1..4, 0);
    println!("{:?}", c);
    let mut c2 = vec![0, 1, 2, 3, 4, 5];
    c2.copy_within(0..3, 3);
    println!("{:?}", c2);
    let mut c3 = vec![0, 1, 2, 3];
    c3.copy_within(1.., 0);
    println!("{:?}", c3);
    let mut c4 = [0u8, 1, 2, 3, 4];
    c4.copy_within(..2, 3);
    println!("{:?}", c4);
    c4.copy_within(0..=1, 1);
    println!("{:?}", c4);
    let r = 1..3;
    c4.copy_within(r, 0);
    println!("{:?}", c4);
    let mut c5 = vec!["a", "b"];
    c5.copy_within(0..1, 1);
    println!("{:?}", c5);
    let mut c6 = vec![0, 1, 2];
    c6.copy_within(3..3, 3);
    println!("{:?}", c6);
    c6[..].copy_within(.., 0);
    println!("{:?}", c6);
}
