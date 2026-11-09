// The slice split family with closures, as core: split, rsplit, splitn,
// rsplitn, split_inclusive and the _mut forms; predicate call order,
// size_hint, and both ends.
fn main() {
    let v = vec![1, 0, 2, 0, 0, 3];
    let mut calls = 0;
    let mut it = v.split(|x| { calls += 1; println!("pred {}", x); *x == 0 });
    println!("{:?}", it.size_hint());
    println!("{:?}", it.next());
    println!("{:?}", it.next_back());
    println!("{:?}", it.size_hint());
    println!("{:?}", it.next());
    println!("{:?}", it.next());
    println!("{:?}", it.next());
    println!("{:?} {:?}", it.next(), it.size_hint());
    drop(it);
    println!("calls {}", calls);
    let e: [i32; 0] = [];
    println!("{:?}", e.split(|x| *x == 0).collect::<Vec<_>>());
    println!("{:?}", e.split_inclusive(|x| *x == 0).collect::<Vec<_>>());
    println!("{:?}", e.split_inclusive(|x| *x == 0).size_hint());
    println!("{:?}", [0, 0].split_inclusive(|x| *x == 0).rev().collect::<Vec<_>>());
    println!("{:?}", [1, 0, 2].split_inclusive(|x| *x == 0).rev().collect::<Vec<_>>());
    println!("{:?}", [1, 0].split_inclusive(|x| *x == 0).size_hint());
    for n in 0..4 {
        println!("{} {:?} {:?}", n, v.splitn(n, |x| *x == 0).collect::<Vec<_>>(), v.rsplitn(n, |x| *x == 0).collect::<Vec<_>>());
        println!("{:?}", v.splitn(n, |x| *x == 0).size_hint());
    }
    println!("{}", v.split(|x| *x == 0).count());
    println!("{:?}", v.split(|x| *x == 0).last());
    println!("{:?}", v.rsplit(|x| *x == 0).nth(1));
    println!("{:?}", v.split(|x| *x > 1).map(|s| s.len()).collect::<Vec<_>>());
    let mut w = vec![3u8, 9, 4, 9, 5];
    for part in w.split_mut(|x| *x == 9) { part[0] += 1; }
    for part in w.rsplitn_mut(2, |x| *x == 4) { part.reverse(); }
    for part in w.split_inclusive_mut(|x| *x == 9).rev() { part[0] *= 2; }
    println!("{:?}", w);
    let s = b"key=value=more";
    let mut kv = s.splitn(2, |&b| b == b'=');
    println!("{:?} {:?} {:?}", kv.next().map(|k| String::from_utf8_lossy(k)), kv.next().map(|k| k.len()), kv.next());
    let words: Vec<String> = b"a  b c".split(|b| *b == b' ').filter(|p| !p.is_empty()).map(|p| String::from_utf8(p.to_vec()).unwrap()).collect();
    println!("{:?}", words);
    let arr = [[1, 2], [0, 0], [3, 4]];
    println!("{:?}", arr.split(|p| p[0] == 0).collect::<Vec<_>>());
    let strs = vec!["a".to_string(), "".to_string(), "b".to_string()];
    let groups: Vec<Vec<&String>> = strs.split(|s| s.is_empty()).map(|g| g.iter().collect()).collect();
    println!("{:?}", groups);
    let a3 = [1, 2, 3];
    let mut it2 = a3.rsplit(|x| *x == 2);
    println!("{:?} {:?} {:?}", it2.next_back(), it2.next(), it2.next());
}
