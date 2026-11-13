// Deref coercions: &Box<String> to &str, &Box<Vec> to a
// slice, &mut Box to &mut, nested boxes.
fn take_str(s: &str) { println!("str {}", s); }
fn take_slice(s: &[i32]) { println!("slice {:?} {}", s, s.len()); }
fn take_mut(x: &mut i32) { *x += 1; }
fn main() {
    let bs = Box::new("boxed".to_string());
    take_str(&bs);
    take_str(&&bs);
    let bv = Box::new(vec![4, 5, 6]);
    take_slice(&bv);
    let mut bm = Box::new(10);
    take_mut(&mut bm);
    println!("{}", bm);
    let nested = Box::new(Box::new(3));
    println!("{}", **nested);
    let bb = Box::new(vec![9]);
    let sl: &[i32] = &bb;
    println!("{:?}", sl);
}
