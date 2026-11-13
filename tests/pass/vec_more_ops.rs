// The second rank of Vec and slice methods: swap_remove,
// reverse (Vec, slice, and array), append, extend_from_slice,
// split_off, resize, dedup, and the mutable element refs.
fn main() {
    let mut v = vec![1, 2, 3, 4];
    println!("{} {:?}", v.swap_remove(1), v);
    v.reverse();
    println!("{:?}", v);
    let mut w = vec![9, 8];
    v.append(&mut w);
    println!("{:?} {} {:?}", v, v.capacity(), w);
    v.extend_from_slice(&[7, 7]);
    println!("{:?} {}", v, v.capacity());
    v.dedup();
    println!("{:?}", v);
    let t = v.split_off(2);
    println!("{:?} {:?} {} {}", v, t, v.capacity(), t.capacity());
    let mut r: Vec<i32> = Vec::new();
    r.resize(3, 5);
    println!("{:?} {}", r, r.capacity());
    r.resize(1, 9);
    println!("{:?}", r);
    if let Some(x) = r.first_mut() {
        *x += 10;
    }
    let none = r.get_mut(9).is_none();
    println!("{:?} {}", r, none);
    let mut a = [5, 6, 7];
    a.reverse();
    println!("{:?} {:?}", a, a.first());
    let mut big = vec![3, 1, 2, 4];
    big.as_mut_slice().reverse();
    let s: &mut [i32] = &mut big[..];
    s.reverse();
    println!("{:?}", big);
    let sl: &[i32] = &big;
    println!("{:?} {:?} {:?} {}", sl.first(), sl.last(), sl.get(2), sl.contains(&2));
    let s2: &[i32] = big.as_slice();
    println!("{:?} {}", s2, s2.len());
}
