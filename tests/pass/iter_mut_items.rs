// iter_mut items stay usable: writes of static strs, enumerate, a
// filtered collect of `&mut`, and a direct `next`.
fn main() {
    let mut v: Vec<&str> = vec!["a", "b"];
    for r in v.iter_mut() {
        *r = "c";
    }
    let mut w = vec![1, 2, 3];
    for (i, x) in w.iter_mut().enumerate() {
        *x += i;
    }
    let t: Vec<&mut usize> = w.iter_mut().filter(|x| **x > 2).collect();
    println!("{:?} {:?}", v, t);
    let mut u: Vec<&'static str> = vec!["x"];
    if let Some(r) = u.iter_mut().next() {
        *r = "y";
    }
    println!("{:?} {:?}", u, w);
}
