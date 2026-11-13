// `[T]::get` and `get_mut` with range forms (tf-slice): syntactic
// ranges, range values of every kind, and `(Bound, Bound)` pairs;
// out-of-range is `None`. Range values as the index in brackets
// make the same slice place.
use std::ops::Bound;
fn main() {
    let v = vec![10, 20, 30, 40];
    println!("{:?} {:?} {:?}", v.get(1), v.get(9), v.get(..2));
    println!("{:?} {:?} {:?}", v.get(1..3), v.get(2..=3), v.get(3..));
    println!("{:?} {:?} {:?}", v.get(..), v.get(..=3), v.get(4..4));
    println!("{:?} {:?} {:?}", v.get(3..2), v.get(0..5), v.get(5..5));
    println!("{:?} {:?}", v.get(0..=4), v.get(..=4));
    println!("{:?} {:?}", v.get(1..3), &v[1..3]);
    let (ra, rb) = (..2, ..2);
    println!("{:?} {:?}", v.get(ra), &v[rb]);
    let (ra, rb) = (2.., 2..);
    println!("{:?} {:?}", v.get(ra), &v[rb]);
    let (ra, rb) = (1..=2, 1..=2);
    println!("{:?} {:?}", v.get(ra), &v[rb]);
    let (ra, rb) = (..=1, ..=1);
    println!("{:?} {:?}", v.get(ra), &v[rb]);
    let (ra, rb) = (.., ..);
    println!("{:?} {:?}", v.get(ra), &v[rb]);
    let rr = 1..3;
    println!("{:?}", v.get(rr));
    println!("{:?}", v.get((Bound::Included(1), Bound::Excluded(3))));
    println!("{:?}", v.get((Bound::Excluded(0), Bound::Included(2))));
    println!("{:?}", v.get((Bound::Unbounded, Bound::Excluded(2))));
    println!("{:?}", v.get((Bound::Excluded(3), Bound::Unbounded)));
    let a = [5, 6, 7];
    println!("{:?} {:?}", a.get(1..), &a[..2]);
    let b: &[u8] = b"abcd";
    let n = 2usize;
    println!("{:?}", b.get(..n));
    let mut m = vec![1, 2, 3, 4];
    if let Some(s) = m.get_mut(1..3) {
        s[0] = 99;
    }
    println!("{:?}", m);
    let rm = 2..=3;
    if let Some(s) = m.get_mut(rm) {
        s[1] = 77;
    }
    println!("{:?}", m);
    m[1..3].copy_from_slice(&[8, 9]);
    println!("{:?}", m);
    let rv = 0..2;
    m[rv].copy_from_slice(&[4, 5]);
    println!("{:?}", m);
    // An exhausted inclusive range slices empty at end + 1.
    let mut ex = 1..=2;
    while ex.next().is_some() {}
    println!("{:?}", v.get(ex));
    let mut ex2 = 1..=2;
    while ex2.next().is_some() {}
    println!("{:?}", &v[ex2]);
    let s = "héllo";
    let sr = ..3;
    println!("{}", &s[sr]);
}
