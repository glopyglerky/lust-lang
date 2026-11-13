// A user IntoIterator whose IntoIter is vec::IntoIter: a for loop
// over the value and an explicit into_iter().collect().
struct Pairs<K, V> {
    entries: Vec<(K, V)>,
}

impl<K, V> IntoIterator for Pairs<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

fn main() {
    let p = Pairs { entries: vec![(1u8, "a"), (2, "b")] };
    for (k, v) in p {
        println!("{} {}", k, v);
    }
    let q = Pairs { entries: vec![(3u8, 'c')] };
    let v: Vec<(u8, char)> = q.into_iter().collect();
    println!("{:?}", v);
}
