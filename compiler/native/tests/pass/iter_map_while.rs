// Iterator::map_while (R22): not fused, (0, upper) size_hint, in-place collect, drops.
#[derive(Debug)]
struct D(i32);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
fn main() {
    let v = [Some(1), Some(2), None, Some(4)];
    let a: Vec<i32> = v.iter().map_while(|x| *x).collect();
    println!("{:?}", a);
    let mut it = v.iter().map_while(|x| x.map(|y| y * 10));
    println!("{:?} {:?}", it.size_hint(), it.next());
    println!("{:?} {:?} {:?} {:?}", it.next(), it.next(), it.next(), it.next());
    let w: Vec<u32> = vec![1, 2, 3, 0, 5];
    let s: u32 = w.iter().map_while(|&x| if x > 0 { Some(x) } else { None }).sum();
    println!("{s}");
    let c = (0..10).map_while(|i| (i < 4).then_some(i * i)).count();
    println!("{c}");
    let strs = vec!["1", "2", "x", "4"];
    let n: Vec<u8> = strs.iter().map_while(|s| s.parse().ok()).collect();
    println!("{:?}", n);
    let ds = vec![D(1), D(2), D(3)];
    let got: Vec<D> = ds.into_iter().map_while(|d| if d.0 < 2 { Some(d) } else { None }).collect();
    println!("got {:?}", got);
    let src: Vec<u32> = (0..100).collect();
    let cap = src.capacity();
    let in_place: Vec<u32> = src.into_iter().map_while(|x| (x < 10).then_some(x + 1)).collect();
    println!("{} {}", in_place.len(), in_place.capacity() == cap);
    let mut e = (1..4).map_while(|x| Some(x)).peekable();
    let pk = e.peek().copied(); println!("{:?} {:?}", pk, e.size_hint());
    let last = (0..5u8).map_while(|x| x.checked_sub(1)).last();
    println!("{:?}", last);
    let mut f = vec![1u8, 2].into_iter().map_while(|x| Some(x as u32));
    let r = &mut f;
    println!("{:?} {:?}", r.next(), f.next());
}
