// Flatten drop timing (W-STD-B): a fused outer iterator (every std
// one) stays until the Flatten drops, as std's Fuse specialization keeps
// it; the front inner iterator drops with the Flatten.
struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D{}", self.0);
    }
}
fn main() {
    let d = D(1);
    let mut it = (0..2).flat_map(move |x| {
        let _ = &d;
        vec![x]
    });
    println!("{:?}", it.next());
    println!("{:?}", it.next());
    println!("{:?}", it.next());
    println!("x");
    println!("{:?}", it.next());
    drop(it);
    println!("y");
    let vv = vec![vec![D(2)], vec![D(3), D(4)]];
    let mut f = vv.into_iter().flatten();
    let a = f.next();
    println!("got {}", match &a { Some(d) => d.0, None => 0 });
    drop(a);
    let b = f.next();
    println!("got {}", match &b { Some(d) => d.0, None => 0 });
    drop(f);
    println!("z");
}
