// flat_map with fn items and fn pointers (ts-rust core.rs:1619 flat_map(Chunk::into_values)).
fn dup(x: u8) -> Vec<u8> { vec![x, x] }
fn opt(x: u8) -> Option<u8> { if x > 1 { Some(x) } else { None } }
fn main() {
    let v = vec![1u8, 2, 3];
    let a: Vec<u8> = v.iter().copied().flat_map(dup).collect();
    let b: Vec<u8> = v.iter().copied().flat_map(opt).collect();
    let fp: fn(u8) -> Vec<u8> = dup;
    let c: Vec<u8> = v.iter().copied().flat_map(fp).collect();
    let s: Vec<char> = vec!["ab", "c"].into_iter().flat_map(str::chars).collect();
    println!("{:?} {:?} {:?} {:?}", a, b, c, s);
}
