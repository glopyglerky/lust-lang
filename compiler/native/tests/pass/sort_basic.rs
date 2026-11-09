// The slice sort family over Vec, slices, and arrays (W-STD-A phase 3):
// stable and unstable sorts, by comparator and by key, and the binary
// searches, including the index Rust 1.99 returns among duplicates.
fn main() {
    let mut v = vec![5, 3, 9, 1, 3, 7, -2];
    v.sort();
    println!("{:?}", v);
    let mut w = vec!["pear", "fig", "apple", "kiwi", "fig"];
    w.sort_unstable();
    println!("{:?}", w);
    let mut a = [9u8, 2, 7, 2];
    a.sort_unstable_by(|x, y| y.cmp(x));
    println!("{:?}", a);
    let mut s = vec![String::from("ccc"), String::from("a"), String::from("bb"), String::from("dd")];
    s.sort_by_key(|x| x.len());
    println!("{:?}", s);
    s.sort_by(|x, y| y.cmp(x));
    println!("{:?}", s);
    let sl = &mut v[2..6];
    sl.sort_by(|x, y| y.cmp(x));
    println!("{:?}", v);
    let pairs = vec![(1, 'a'), (2, 'b'), (2, 'c'), (2, 'd'), (5, 'e'), (8, 'f')];
    println!("{:?} {:?} {:?}", pairs.binary_search(&(2, 'c')), pairs.binary_search(&(3, 'x')), pairs.binary_search(&(9, 'a')));
    println!("{:?}", pairs.binary_search_by_key(&2, |p| p.0));
    println!("{:?}", pairs.binary_search_by(|p| p.0.cmp(&0)));
    let dup = vec![4; 9];
    println!("{:?}", dup.binary_search(&4));
    let e: Vec<i32> = Vec::new();
    println!("{:?}", e.binary_search(&1));
    let mut words = vec!["Bb", "aa", "C", "bb"];
    words.sort_by_key(|w| w.to_lowercase());
    println!("{:?}", words);
    let mut f = vec![2.5f64, -1.0, 9.75, 0.0];
    f.sort_by(|x, y| x.partial_cmp(y).unwrap());
    println!("{:?}", f);
}
