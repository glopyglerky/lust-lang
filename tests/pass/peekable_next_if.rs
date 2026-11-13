// Peekable::next_if and next_if_eq (R22): put-back on a miss, the closure drops before the call returns, PartialEq order.
#[derive(Debug)]
struct D(u32);
impl Drop for D { fn drop(&mut self) { println!("drop D{}", self.0); } }
#[derive(Debug)]
struct Q(i32);
impl PartialEq for Q { fn eq(&self, o: &Q) -> bool { println!("eq {} {}", self.0, o.0); self.0 == o.0 } }
fn main() {
    let mut p = vec![1u32, 2, 5].into_iter().peekable();
    let a = p.next_if(|x| *x == 1);
    let b = p.next_if_eq(&2);
    let c = p.peek().copied();
    let d = p.next_if_eq(&9);
    let e = p.next_if(|_| false);
    println!("{a:?} {b:?} {c:?} {d:?} {e:?} {:?}", p.collect::<Vec<_>>());
    let v = vec![1, 2, 3];
    let mut q = v.iter().peekable();
    let a = q.next_if(|x| **x == 1);
    let b = q.next_if_eq(&&2);
    println!("{a:?} {b:?} {:?}", q.next());
    let mut r = vec![D(1), D(2)].into_iter().peekable();
    let tok = D(10);
    let x = r.next_if(move |d| { let _k = &tok; d.0 == 7 });
    println!("x {:?}", x);
    let y = r.next_if(|d| d.0 == 1);
    println!("y {:?}", y);
    drop(r);
    let mut s = vec![Q(1), Q(2)].into_iter().peekable();
    println!("{:?}", s.next_if_eq(&Q(1)));
    println!("{:?}", s.next_if_eq(&Q(3)));
    let mut e = Vec::<u8>::new().into_iter().peekable();
    println!("{:?} {:?}", e.next_if(|_| true), e.peek());
    let words = ["a", "b"];
    let mut w = words.iter().copied().peekable();
    println!("{:?}", w.next_if_eq(&"a"));
    let mut cs = "aab".chars().peekable();
    let mut n = 0;
    while cs.next_if_eq(&'a').is_some() { n += 1; }
    println!("{n} {:?}", cs.next());
    let mut it = (0..10).peekable();
    let r2 = &mut it;
    while let Some(x) = r2.next_if(|&x| x < 3) { print!("{x} "); }
    println!("{:?}", it.size_hint());
}
