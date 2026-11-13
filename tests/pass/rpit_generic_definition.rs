// A generic fn returning `impl Trait`: the template check lowers its return
// local as the hidden type, so the borrow checker sees concrete regions (W-CHECK-G3).
struct M<K, V> {
    entries: Vec<(K, V)>,
}

impl<K: Clone, V> M<K, V> {
    fn keys_owned(&self) -> impl Iterator<Item = K> {
        let ks: Vec<K> = self.entries.iter().map(|e| e.0.clone()).collect();
        ks.into_iter()
    }
    fn len(&self) -> usize {
        self.entries.len()
    }
}

fn main() {
    let m = M { entries: vec![(1u8, 'a')] };
    println!("{}", m.len());
}
