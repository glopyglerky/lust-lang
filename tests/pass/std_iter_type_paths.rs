// Std iterator types named by path: in returns, fields, and adapter
// stacks; a `Map` or `FilterMap` over a fn pointer.
struct Cursor<'a> {
    it: std::slice::Iter<'a, u8>,
}

fn evens(v: &[u32]) -> std::vec::IntoIter<u32> {
    let out: Vec<u32> = v.iter().copied().filter(|x| x % 2 == 0).collect();
    out.into_iter()
}

fn first<'a>(v: &'a [u8]) -> std::slice::Iter<'a, u8> {
    v.iter()
}

fn chars_of(s: &str) -> std::str::Chars<'_> {
    s.chars()
}

fn down(n: u32) -> std::iter::Rev<std::ops::Range<u32>> {
    (0..n).rev()
}

fn numbered(v: &[char]) -> std::iter::Enumerate<std::iter::Copied<std::slice::Iter<'_, char>>> {
    v.iter().copied().enumerate()
}

fn plus_one(x: &u8) -> u16 {
    *x as u16 + 1
}

fn halves(x: &u8) -> Option<u8> {
    if x % 2 == 0 { Some(x / 2) } else { None }
}

fn mapped(v: &[u8]) -> std::iter::Map<std::slice::Iter<'_, u8>, fn(&u8) -> u16> {
    v.iter().map(plus_one as fn(&u8) -> u16)
}

fn filtered(v: &[u8]) -> std::iter::FilterMap<std::slice::Iter<'_, u8>, fn(&u8) -> Option<u8>> {
    v.iter().filter_map(halves as fn(&u8) -> Option<u8>)
}

fn main() {
    let total: u32 = evens(&[1, 2, 3, 4]).sum();
    let v = vec![5u8, 6, 8];
    let mut it = first(&v);
    let a = it.next();
    let n = chars_of("héllo").count();
    let d: Vec<u32> = down(4).collect();
    let e: Vec<(usize, char)> = numbered(&['x', 'y']).collect();
    let m: Vec<u16> = mapped(&v).collect();
    let f: Vec<u8> = filtered(&v).collect();
    let mut c = Cursor { it: v.iter() };
    let skip = c.it.next();
    let rest: Vec<&u8> = c.it.collect();
    println!("{} {:?} {} {:?} {:?}", total, a, n, d, e);
    println!("{:?} {:?} {:?} {:?}", m, f, skip, rest);
}
