// In-place collect (W-STD-B): collecting a vec::IntoIter, or std's
// InPlaceIterable adapters over one, keeps the source buffer, so the
// capacity is the source's (in bytes) when the alignments match.
fn main() {
    let mk = || { let mut v: Vec<i64> = Vec::with_capacity(10); v.extend([1, 2, 3, 4, 5]); v };
    let a: Vec<i64> = mk().into_iter().map(|x| x + 1).collect();
    let b: Vec<i64> = mk().into_iter().filter(|x| x % 2 == 0).collect();
    let c: Vec<i64> = mk().into_iter().filter_map(|x| if x > 2 { Some(x) } else { None }).collect();
    let d: Vec<i64> = mk().into_iter().skip(2).collect();
    let e: Vec<i64> = mk().into_iter().take(2).collect();
    let f: Vec<i32> = mk().into_iter().map(|x| x as i32).collect();
    let g: Vec<(usize, i64)> = mk().into_iter().enumerate().collect();
    let h: Vec<i64> = mk().into_iter().rev().collect();
    let i: Vec<u8> = mk().into_iter().map(|x| x as u8).collect();
    let j: Vec<i64> = mk().into_iter().take_while(|x| *x < 3).collect();
    let k: Vec<i64> = mk().into_iter().skip_while(|x| *x < 3).collect();
    let l: Vec<i64> = mk().into_iter().peekable().collect();
    let m: Vec<i64> = mk().into_iter().inspect(|_| {}).collect();
    let n: Vec<i64> = mk().into_iter().step_by(2).collect();
    let o: Vec<[i64; 2]> = mk().into_iter().map(|x| [x, x]).collect();
    println!("{} {} {} {} {} {} {} {} {} {} {} {} {} {} {}", a.capacity(), b.capacity(), c.capacity(), d.capacity(), e.capacity(), f.capacity(), g.capacity(), h.capacity(), i.capacity(), j.capacity(), k.capacity(), l.capacity(), m.capacity(), n.capacity(), o.capacity());
}
