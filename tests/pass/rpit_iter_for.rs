// impl Iterator returns in for loops and next(): a &self method returning impl Iterator + '_, and a range (W-CHECK-G3).
struct C { v: Vec<u32> }
impl C { fn items(&self) -> impl Iterator<Item = &u32> + '_ { self.v.iter() } }
fn nums(n: u32) -> impl Iterator<Item = u32> { 0..n }
fn main() {
    let c = C { v: vec![1, 2, 3, 4, 6] };
    let mut s = 0;
    for x in c.items() { s += *x; }
    println!("{}", s);
    for i in nums(3) { println!("{}", i); }
    let mut it = nums(2);
    println!("{:?} {:?} {:?}", it.next(), it.next(), it.next());
}
