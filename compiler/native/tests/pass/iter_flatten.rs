// `flatten` and `flat_map` (W-STD-B), as std's FlattenCompat: inner
// Vecs, slices, arrays, Options, and iterators; size_hint (and so
// collect's capacity) as std computes it, with the fixed-size form for
// arrays.
fn main() {
    let vv = vec![vec![1, 2], vec![], vec![3]];
    let flat: Vec<i32> = vv.iter().flatten().copied().collect();
    println!("{flat:?} {}", flat.capacity());
    let owned: Vec<i32> = vv.clone().into_iter().flatten().collect();
    println!("{owned:?} {}", owned.capacity());
    let opts = vec![Some(1), None, Some(3), None];
    let somes: Vec<i32> = opts.iter().copied().flatten().collect();
    println!("{somes:?} {}", somes.capacity());
    let arrs = vec![[1u8, 2], [3, 4], [5, 6]];
    let fa: Vec<u8> = arrs.iter().flatten().copied().collect();
    let fo: Vec<u8> = arrs.clone().into_iter().flatten().collect();
    println!("{fa:?} {} {fo:?} {}", fa.capacity(), fo.capacity());
    let fm: Vec<u32> = (1..4u32).flat_map(|n| vec![n; n as usize]).collect();
    println!("{fm:?} {}", fm.capacity());
    let fma: Vec<u32> = (1..4u32).flat_map(|n| [n, n * 10]).collect();
    println!("{fma:?} {}", fma.capacity());
    let words = vec!["ab", "c"];
    let chars: Vec<char> = words.iter().flat_map(|w| w.chars()).collect();
    println!("{chars:?}");
    let mut it = vv.iter().flatten();
    println!("{:?} {:?}", it.size_hint(), it.next());
    println!("{:?}", it.size_hint());
    let n = vv.iter().flatten().count();
    let s: i32 = vv.iter().flatten().sum();
    println!("{n} {s} {:?}", vv.iter().flatten().max());
    let mut calls = 0;
    let firsts: Vec<i32> = (0..5).flat_map(|x| {
        calls += 1;
        if x % 2 == 0 { Some(x) } else { None }
    }).take(2).collect();
    println!("{firsts:?} {calls}");
}
