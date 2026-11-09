// Seeded random inputs through the W-STD-B surface (entry closure
// forms, flatten and flat_map, windows, char_indices, rposition, user
// iterators with adapters, From/Into, Cow views), compared with rustc.
use std::borrow::Cow;
use std::collections::HashMap;

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

struct Steps {
    left: u32,
    at: u32,
}

impl Iterator for Steps {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.left == 0 {
            return None;
        }
        self.left -= 1;
        self.at = self.at * 3 % 17;
        Some(self.at)
    }
}

fn word(r: &mut Lcg) -> String {
    let n = r.below(5) as usize;
    let alphabet = ['a', 'b', 'é', 'ß', 'z'];
    (0..n).map(|_| alphabet[r.below(5) as usize]).collect()
}

fn label<'a>(s: &'a str, owned: bool) -> Cow<'a, str> {
    if owned { Cow::Owned(s.to_uppercase()) } else { Cow::Borrowed(s) }
}

fn main() {
    let mut r = Lcg(42);
    for round in 0..6 {
        // Entry closure forms.
        let mut m: HashMap<u64, Vec<u64>> = HashMap::new();
        let mut made = 0;
        for _ in 0..12 {
            let k = r.below(5);
            let v = r.below(100);
            m.entry(k)
                .and_modify(|xs| xs.push(v))
                .or_insert_with(|| {
                    made += 1;
                    vec![v]
                });
        }
        let mut ks: Vec<(&u64, &Vec<u64>)> = m.iter().collect();
        ks.sort();
        println!("{round} entries {ks:?} made {made}");
        // flatten / flat_map with size hints.
        let vv: Vec<Vec<u64>> = (0..r.below(4)).map(|i| (0..r.below(3)).map(|j| i * 10 + j).collect::<Vec<u64>>()).collect();
        let flat: Vec<u64> = vv.iter().flatten().copied().collect();
        let fm: Vec<u64> = vv.iter().flat_map(|v| v.iter().map(|x| x + 1)).collect();
        println!("flat {flat:?} {} fm {fm:?} {:?}", flat.capacity(), vv.iter().flatten().size_hint());
        // windows and rposition.
        let xs: Vec<u64> = (0..r.below(7)).map(|_| r.below(4)).collect();
        let w = 1 + r.below(3) as usize;
        let sums: Vec<u64> = xs.windows(w).map(|p| p.iter().sum::<u64>()).collect();
        println!("xs {xs:?} w {w} sums {sums:?} last2 {:?}", xs.iter().rposition(|&x| x == 2));
        // char_indices and bytes over random words.
        let s = word(&mut r) + &word(&mut r);
        let idx: Vec<(usize, char)> = s.char_indices().collect();
        println!("s {s:?} idx {idx:?} bytes {} back {:?}", s.bytes().count(), s.char_indices().next_back());
        // A user iterator through adapters.
        let st = Steps { left: r.below(6) as u32, at: 1 + r.below(5) as u32 };
        let evens: Vec<u32> = st.filter(|x| x % 2 == 0).map(|x| x * 2).collect();
        let total: u32 = Steps { left: 4, at: 2 }.zip(Steps { left: 3, at: 5 }).map(|(a, b)| a + b).sum();
        println!("evens {evens:?} {} total {total}", evens.capacity());
        // From/Into and Cow views.
        let t: String = s.as_str().into();
        let bytes: Vec<u8> = t.clone().into();
        let c = label(&t, round % 2 == 0);
        println!("into {} {} cow {} {} {}", t.len(), bytes.len(), c, c.len(), c == t.as_str());
    }
}
