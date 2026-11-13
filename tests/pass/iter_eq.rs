// Iterator::eq and ne as core 1.99: two TrustedLen sides of unequal
// length are unequal with no pull; else one b.next() per item of a; the
// other iterator drops before self; user iterators compare too.
struct D(u32);
impl Drop for D { fn drop(&mut self) { println!("drop D{}", self.0); } }
impl PartialEq for D { fn eq(&self, o: &D) -> bool { println!("cmp {} {}", self.0, o.0); self.0 == o.0 } }
#[derive(Clone, Copy, PartialEq, Debug)]
struct Node(u32);
struct Kids { v: Vec<Node>, i: usize }
impl Iterator for Kids {
    type Item = Node;
    fn next(&mut self) -> Option<Node> { let r = self.v.get(self.i).copied(); self.i += 1; println!("kid pull {:?}", r); r }
}
fn main() {
    println!("{}", [1, 2, 3].iter().eq([1, 2, 3].iter()));
    println!("{}", [1, 2, 3].iter().eq([1, 2].iter()));
    println!("{}", vec![1u8, 2].into_iter().ne(vec![1u8, 3]));
    println!("{}", (0..5).eq(0..5));
    let mut pulls = 0;
    let r = (0..5).inspect(|_| pulls += 1).eq((0..3).inspect(|_| ()));
    println!("{} pulls {}", r, pulls);
    let mut pa = 0;
    let r = [1, 2, 3].iter().filter(|_| { pa += 1; true }).eq([1, 9, 3].iter());
    println!("{} pa {}", r, pa);
    let a = vec![D(1), D(2), D(3)];
    let b = vec![D(1), D(5), D(3), D(4)];
    println!("start");
    let r = a.into_iter().eq(b);
    println!("eq {}", r);
    let k = Kids { v: vec![Node(1), Node(2)], i: 0 };
    println!("{}", k.eq(vec![Node(1), Node(2)].into_iter()));
    let k2 = Kids { v: vec![Node(1), Node(2)], i: 0 };
    println!("{}", k2.eq(vec![Node(1)]));
    let words = ["a", "bb"];
    println!("{}", words.iter().eq(["a", "bb"].iter()));
    let ss = vec![String::from("x")];
    println!("{}", ss.iter().eq(vec![String::from("x")].iter()));
    println!("{}", "abc".chars().eq("abd".chars()));
    println!("{}", [1.0f64, f64::NAN].iter().eq([1.0, f64::NAN].iter()));
}
