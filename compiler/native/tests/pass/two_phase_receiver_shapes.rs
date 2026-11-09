// Two-phase receivers rustc accepts: the argument reads the receiver's
// own place (or a place under it) before the call activates the `&mut`.
struct S { n: i32, v: Vec<i32> }
impl S {
    fn add(&mut self, n: i32) { self.n += n; }
}
fn main() {
    let mut v = vec![1, 2, 3];
    v.push(v[0]);
    v.truncate(v.len() - 1);
    let r = &mut v;
    r.push(r[0]);
    r.push(r.len() as i32);
    let mut s = S { n: 1, v: vec![1] };
    s.add(s.n);
    s.v.push(s.n);
    s.v.push(s.v[0]);
    let q = &mut s;
    q.add(q.n);
    q.v.push(q.v[0]);
    let mut t = (vec![1], 2);
    let rt = &mut t;
    rt.0.push(rt.1);
    let mut b = Box::new(vec![1]);
    b.push(b[0]);
    let sl: &mut [i32] = &mut v;
    if let Some(x) = sl.get_mut(sl.len() - 1) {
        *x = 7;
    }
    println!("{:?} {} {:?} {:?} {:?}", v, s.n, s.v, t, b);
}
