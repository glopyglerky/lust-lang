// `Ord::max`/`min`/`clamp` on a receiver that is itself `Ord` (`&A`,
// `&&A`, `&mut A`, `Box<A>`): Self is the receiver type, so the
// arguments and the result have it too. Floats keep their inherent
// `max`/`min` through the deref.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct S(u8);
impl Drop for S {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let a = &3;
    let b = &5;
    println!("{} {} {}", a.max(b), a.min(b), (&9).clamp(&1, &5));
    let (x, y) = (4, 6);
    let rx = &&x;
    let ry = &&y;
    println!("{}", rx.max(ry));
    let f = &1.5f64;
    println!("{} {}", f.max(2.0), f.min(0.5));
    println!("{} {}", Box::new(3).max(Box::new(7)), Box::new(9u8).min(Box::new(2)));
    let (mut m, mut n) = (3, 8);
    let big = (&mut m).max(&mut n);
    *big += 1;
    println!("{} {}", m, n);
    let v = vec![3, 1, 4];
    println!("{}", (&v[0]).max(&v[2]));
    let (s1, s2) = (String::from("b"), String::from("a"));
    println!("{}", (&s1).min(&s2));
    println!("{:?}", (&Some(3)).max(&None));
    let (p, q) = (S(1), S(2));
    {
        let w = (&p).max(&q);
        println!("{:?} {:?}", w, (&p).clamp(&p, &q));
    }
    println!("end");
}
