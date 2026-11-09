// Vec methods as values (ts-rust effect/graph.rs:388 map(Vec::as_slice)).
fn main() {
    let m: Option<&Vec<u32>> = None;
    let v = vec![1u32, 2];
    let a: &[u32] = m.map(Vec::as_slice).unwrap_or(&[]);
    let b: &[u32] = Some(&v).map(Vec::as_slice).unwrap_or(&[]);
    let vs = vec![vec![1u8], vec![], vec![2, 3]];
    let lens: Vec<usize> = vs.iter().map(Vec::len).collect();
    let empties = vs.iter().filter(|x| Vec::is_empty(x)).count();
    let f = |x: &Vec<u8>| -> Vec<u8> { x.clone() };
    let mut w = vec![5u16, 6];
    let ms: &mut [u16] = Vec::as_mut_slice(&mut w);
    ms[0] = 7;
    println!("{:?} {:?} {:?} {} {:?} {:?}", a, b, lens, empties, f(&vs[2]), w);
}
