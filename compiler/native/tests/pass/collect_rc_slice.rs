// collect into Rc<[T]>, Arc<[T]>, and Box<str> (alloc's FromIterator impls):
// contents, the order of side effects, drop order, and counts.
use std::cell::OnceCell;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug)]
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D{}", self.0);
    }
}

type Chunk = Rc<[OnceCell<u32>]>;

fn words(s: &str) -> Rc<[&str]> {
    s.split(' ').filter(|w| !w.is_empty()).collect()
}

fn main() {
    // TrustedLen sources: one allocation of the exact length.
    let a: Rc<[u32]> = (0..4).collect();
    let b = (0..3).map(|i| { println!("map {i}"); format!("s{i}") }).collect::<Arc<[String]>>();
    let c: Rc<[D]> = vec![D(1), D(2), D(3)].into_iter().take(2).collect();
    println!("{a:?} {b:?} {c:?}");
    // Other sources: a Vec collect, then `From<Vec<T>>`.
    let d: Arc<[D]> = vec![D(4), D(5), D(6)].into_iter().filter(|d| d.0 != 5).collect();
    let e = Rc::<[u8]>::from_iter([3u8, 1, 2].into_iter().filter(|&x| x > 1));
    println!("{d:?} {e:?}");
    let f: Box<str> = "h\u{e9}llo".chars().rev().collect();
    let g: Box<str> = vec!["a", "b", "c"].into_iter().collect();
    let h: Box<str> = (0..3).map(|i| i.to_string()).collect();
    println!("{f} {g} {h}");
    let seed: Vec<Option<Vec<u32>>> = vec![Some(vec![7, 8]), None];
    let datas: Vec<Option<Chunk>> = seed.into_iter().map(|c| c.map(|n| n.into_iter().map(|x| OnceCell::from(x)).collect::<Chunk>())).collect();
    let first = datas[0].clone().unwrap();
    println!("{:?} {} {}", first[1].get(), Rc::strong_count(&first), datas[1].is_none());
    let text = String::from("one  two three");
    println!("{:?}", words(&text));
    let empty: Arc<[D]> = Vec::new().into_iter().collect();
    println!("{}", empty.len());
    drop(c);
    drop(d);
    println!("end");
}
