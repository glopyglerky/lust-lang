// `clone_from` in a written Clone impl (R24, the ts-rust
// NodeArena shape), derived types taking the default, generics,
// and the qualified `Clone::clone_from` form.
use std::rc::Rc;
#[derive(Clone, Debug)]
enum E { A(Vec<i32>), B(String), C }
#[derive(Clone, Debug)]
struct S { v: Vec<i32>, e: E, t: (String, Option<Vec<u8>>) }

#[derive(Debug)]
struct Arena { id: u64, nodes: Vec<String>, src: Vec<u8> }
impl Clone for Arena {
    fn clone(&self) -> Self { Arena { id: self.id + 100, nodes: self.nodes.clone(), src: self.src.clone() } }
    fn clone_from(&mut self, source: &Self) {
        self.id = source.id + 100;
        self.nodes.clone_from(&source.nodes);
        self.src.clone_from(&source.src);
    }
}
#[derive(Clone, Debug)]
struct Outer { a: Arena, n: i32 }
fn generic<T: Clone>(a: &mut T, b: &T) { a.clone_from(b); }
fn qualified(a: &mut Vec<i32>, b: &Vec<i32>) { Clone::clone_from(a, b); }
fn main() {
    let mut s = S { v: Vec::with_capacity(50), e: E::A(vec![1]), t: ("x".to_string(), None) };
    s.v.push(1);
    let s2 = S { v: vec![7, 8], e: E::B("hi".to_string()), t: ("y".to_string(), Some(vec![1, 2])) };
    s.clone_from(&s2);
    println!("{:?} cap={}", s, s.v.capacity());
    s.clone_from(&S { v: vec![], e: E::C, t: (String::new(), None) });
    println!("{:?}", s);
    let mut o = Some(vec![1, 2, 3]);
    o.clone_from(&None);
    println!("{:?}", o);
    o.clone_from(&Some(vec![9]));
    println!("{:?}", o);
    let mut r: Result<Vec<i32>, String> = Ok(vec![1]);
    r.clone_from(&Err("e".to_string()));
    println!("{:?}", r);
    let mut arr = [vec![1], vec![2]];
    arr.clone_from(&[vec![3], vec![4, 5]]);
    println!("{:?}", arr);
    let mut rc = Rc::new(vec![1]);
    let rc2 = Rc::new(vec![2, 3]);
    rc.clone_from(&rc2);
    println!("{:?} strong={}", rc, Rc::strong_count(&rc));
    let mut g = vec![1];
    generic(&mut g, &vec![5, 6]);
    println!("{:?}", g);
    let mut gs = S { v: vec![], e: E::C, t: (String::new(), None) };
    generic(&mut gs, &s2);
    println!("{:?}", gs);
    let mut q = vec![0];
    qualified(&mut q, &vec![3, 4]);
    println!("{:?}", q);
    // census shape below uses top-level Arena
    let mut a1 = Arena { id: 1, nodes: vec!["a".to_string()], src: vec![1] };
    let a2 = Arena { id: 2, nodes: vec!["b".to_string(), "c".to_string()], src: vec![2, 3] };
    a1.clone_from(&a2);
    println!("{:?}", a1);
    let a3 = a2.clone();
    println!("{:?}", a3);
    // derived outer over manual inner: field clone_from must run
    let mut o1 = Outer { a: Arena { id: 0, nodes: vec![], src: vec![] }, n: 0 };
    o1.clone_from(&Outer { a: Arena { id: 5, nodes: vec!["z".to_string()], src: vec![9] }, n: 3 });
    println!("{:?}", o1);
}
