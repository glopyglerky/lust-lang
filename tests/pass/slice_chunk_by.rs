// [T]::chunk_by and chunk_by_mut (R13): runs from both ends, predicate calls, size_hint, last as next_back, count.
fn main() {
    let v = [1, 1, 2, 3, 3, 3, 4, 1, 1];
    let g: Vec<&[i32]> = v.chunk_by(|a, b| a == b).collect();
    println!("{:?}", g);
    let r: Vec<&[i32]> = v.chunk_by(|a, b| a <= b).rev().collect();
    println!("{:?}", r);
    let mut it = v.chunk_by(|a, b| { println!("p {a} {b}"); a == b });
    println!("{:?}", it.size_hint());
    println!("{:?}", it.next());
    println!("{:?}", it.next_back());
    println!("{:?}", it.size_hint());
    println!("{:?}", it.next());
    println!("{:?}", v.chunk_by(|a, b| { println!("q {a} {b}"); a == b }).last());
    println!("{}", v.chunk_by(|a, b| { println!("c {a} {b}"); a < b }).count());
    let mut w = vec![1, 1, 2, 3, 3, 3];
    let groups: Vec<&mut [i32]> = w.chunk_by_mut(|a, b| a == b).collect();
    println!("{}", groups.len());
    for g in w.chunk_by_mut(|a, b| a == b) { let n = g.len() as i32; g[0] = n * 10; }
    println!("{:?}", w);
    if let Some(last) = w.chunk_by_mut(|a, b| a != b).last() { last[0] = -1; }
    println!("{:?}", w);
    let e: &[u8] = &[];
    println!("{:?} {:?}", e.chunk_by(|_, _| true).next(), e.chunk_by(|_, _| true).size_hint());
    let s = vec![String::from("a"), String::from("ab"), String::from("b")];
    let by_first: Vec<Vec<&str>> = s.chunk_by(|x, y| x.as_bytes()[0] == y.as_bytes()[0]).map(|c| c.iter().map(|x| x.as_str()).collect()).collect();
    println!("{:?}", by_first);
    let mut steps = 0;
    let n = [5, 4, 3, 9].chunk_by(|a, b| { steps += 1; a > b }).nth(1);
    println!("{:?} {}", n, steps);
}
