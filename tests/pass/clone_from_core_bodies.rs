// `Clone::clone_from` core bodies (R24): Vec and String keep
// their capacity, elements clone_from in place, boxed slices and
// Box follow std, side effects in std's order.
#[derive(Debug)]
struct E(i32);
impl Clone for E {
    fn clone(&self) -> Self { print!("C{} ", self.0); E(self.0) }
    fn clone_from(&mut self, o: &Self) { print!("F{}>{} ", self.0, o.0); self.0 = o.0; }
}
impl Drop for E { fn drop(&mut self) { print!("D{} ", self.0); } }
fn main() {
    // Vec clone_from: prefix F, tail C, truncate drops?
    let mut v = vec![E(1), E(2)];
    let w = vec![E(7), E(8), E(9)];
    v.clone_from(&w);
    println!("-> {:?} cap={}", v.iter().map(|e| e.0).collect::<Vec<_>>(), v.capacity());
    let mut v2 = vec![E(1), E(2), E(3)];
    v2.clone_from(&vec![E(7)]);
    println!("-> {:?} cap={}", v2.iter().map(|e| e.0).collect::<Vec<_>>(), v2.capacity());
    // capacities
    let mut a: Vec<i32> = Vec::with_capacity(100); a.extend([1,2,3]);
    a.clone_from(&vec![9; 5]); println!("cap a={} len={}", a.capacity(), a.len());
    let mut b: Vec<i32> = Vec::with_capacity(4); b.extend([1,2,3,4]);
    b.clone_from(&vec![9; 10]); println!("cap b={} len={}", b.capacity(), b.len());
    let mut c: Vec<i32> = Vec::new();
    c.clone_from(&vec![9; 10]); println!("cap c={} len={}", c.capacity(), c.len());
    // String capacities
    let mut s = String::with_capacity(100); s.push_str("abc");
    s.clone_from(&"hello".to_string()); println!("scap={} {:?}", s.capacity(), s);
    let mut s2 = String::with_capacity(4); s2.push_str("abcd");
    s2.clone_from(&"0123456789".to_string()); println!("s2cap={} {:?}", s2.capacity(), s2);
    let mut s3 = String::new();
    s3.clone_from(&"0123456789".to_string()); println!("s3cap={} {:?}", s3.capacity(), s3);
    // Box<[E]> clone_from
    let mut bs: Box<[E]> = vec![E(1), E(2)].into_boxed_slice();
    let bs2: Box<[E]> = vec![E(7), E(8)].into_boxed_slice();
    bs.clone_from(&bs2);
    println!("-> bs {:?}", bs.iter().map(|e| e.0).collect::<Vec<_>>());
    let bs3: Box<[E]> = vec![E(4)].into_boxed_slice();
    bs.clone_from(&bs3);
    println!("-> bs {:?}", bs.iter().map(|e| e.0).collect::<Vec<_>>());
    // Box<E>
    let mut bx = Box::new(E(1));
    bx.clone_from(&Box::new(E(5)));
    println!("-> bx {}", bx.0);
}
