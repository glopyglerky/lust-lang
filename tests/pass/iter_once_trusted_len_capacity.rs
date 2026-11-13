// std's Once and OnceWith are TrustedLen: collect and extend reserve
// the exact count. FromFn and Successors are not (amortized growth).
fn main() {
    let a: Vec<u8> = std::iter::once(1).collect();
    let b: Vec<u8> = std::iter::once(1).chain(std::iter::once(2)).collect();
    let c: Vec<u8> = std::iter::once_with(|| 3).map(|x| x + 1).collect();
    let d: Vec<u32> = std::iter::from_fn({
        let mut n = 0;
        move || {
            n += 1;
            if n < 5 { Some(n) } else { None }
        }
    })
    .collect();
    let mut e: Vec<u8> = Vec::new();
    e.extend(std::iter::once(7));
    let e1 = e.capacity();
    e.extend(std::iter::once(8));
    let f: Vec<u8> = (0..3u8).chain(std::iter::once(9)).collect();
    println!("{} {} {} {} {} {} {}", a.capacity(), b.capacity(), c.capacity(), d.capacity(), e1, e.capacity(), f.capacity());
    println!("{:?} {:?}", std::iter::once(1).size_hint(), std::iter::once(1).chain(0..4).size_hint());
    let g: Vec<u64> = std::iter::once(1).rev().take(5).zip(std::iter::once_with(|| 2)).map(|(x, y)| x + y).collect();
    let h: Vec<i64> = std::iter::successors(Some(1i64), |x| if *x < 20 { Some(x * 3) } else { None }).collect();
    let i: Vec<u16> = std::iter::once(5).peekable().enumerate().map(|(i, x)| x + i as u16).collect();
    println!("{:?} {} {:?} {} {:?} {}", g, g.capacity(), h, h.capacity(), i, i.capacity());
    let mut j: Vec<u32> = vec![1];
    j.extend(std::iter::once_with(|| 2).chain(std::iter::once(3)));
    println!("{:?} {}", j, j.capacity());
}
