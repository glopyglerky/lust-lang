// More std iterator types named by path: the `str` family, hash
// iterators, `Windows`, `Flatten`, and the option/result iterators, in
// fields, returns, and let annotations. `CharIndices` used to panic
// `lust check`.
use std::collections::HashMap;

struct Cursor<'a> {
    idx: std::str::CharIndices<'a>,
    bytes: std::str::Bytes<'a>,
}

fn lines_of(s: &str) -> std::str::Lines<'_> {
    s.lines()
}

fn split_of(s: &str) -> std::str::Split<'_, char> {
    s.split(',')
}

fn keys_of<'a>(m: &'a HashMap<i32, i32>) -> std::collections::hash_map::Keys<'a, i32, i32> {
    m.keys()
}

fn type_only<T>() {}

fn main() {
    type_only::<std::str::CharIndices>();
    type_only::<std::sync::mpsc::Iter<i32>>();

    let mut c = Cursor { idx: "hé".char_indices(), bytes: "ab".bytes() };
    println!("{:?} {:?}", c.idx.next(), c.bytes.next());

    let ls: Vec<&str> = lines_of("a\nb").collect();
    let sp: Vec<&str> = split_of("x,y").collect();
    println!("{:?} {:?}", ls, sp);

    let mut m = HashMap::new();
    m.insert(7, 8);
    let ksum: i32 = keys_of(&m).sum();
    let it: std::collections::hash_map::Iter<i32, i32> = m.iter();
    println!("{} {}", ksum, it.count());

    let v = [1, 2, 3];
    let w: std::slice::Windows<i32> = v.windows(2);
    let flat: std::iter::Flatten<std::vec::IntoIter<Vec<i32>>> =
        vec![vec![1], vec![2, 3]].into_iter().flatten();
    let o = Some(4);
    let oi: std::option::Iter<i32> = o.iter();
    let ri: std::result::IntoIter<i32> = Ok::<i32, i32>(5).into_iter();
    println!("{} {:?} {} {}", w.count(), flat.collect::<Vec<i32>>(), oi.count(), ri.count());
}
