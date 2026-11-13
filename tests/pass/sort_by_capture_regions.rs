// A sort comparator or key fn that borrows a local: its captures do not
// flow into the slice's elements (L0306 before, in plain fns and in a
// generic instance with T = &str).
fn by_len(v: &mut Vec<&str>) {
    let k = 1;
    v.sort_by(|a, b| (a.len() + k).cmp(&(b.len() + k)));
}

fn by_key(v: &mut Vec<&str>) {
    let k = 1;
    v.sort_by_key(|a| a.len() + k);
}

fn by_unstable(v: &mut [&str]) {
    let k = 1;
    v.sort_unstable_by(|a, b| (a.len() + k).cmp(&(b.len() + k)));
}

fn by<T, K: Ord>(v: &mut Vec<T>, key: impl Fn(&T) -> K) {
    v.sort_by(|a, b| key(a).cmp(&key(b)));
}

fn by_key_generic<T>(v: &mut Vec<T>, key: impl Fn(&T) -> usize) {
    v.sort_by_key(|a| key(a));
}

fn pick<'a>(v: &mut Vec<&'a str>, s: &'a str) {
    v.sort_by_key(|x| if x.len() > 1 { *x } else { s });
    v.push(s);
}

fn main() {
    let mut v = vec!["ccc", "a", "bb"];
    by_len(&mut v);
    println!("{:?}", v);
    let mut v = vec!["ccc", "a", "bb"];
    by_key(&mut v);
    println!("{:?}", v);
    let mut v = vec!["ccc", "a", "bb"];
    by_unstable(&mut v);
    println!("{:?}", v);
    let mut v = vec!["ccc", "a"];
    by(&mut v, |s| s.len());
    println!("{:?}", v);
    let mut v = vec!["ccc", "a"];
    by_key_generic(&mut v, |s| s.len());
    println!("{:?}", v);
    let mut v = vec!["bb", "a"];
    pick(&mut v, "c");
    println!("{:?}", v);
    let mut w: Vec<&str> = vec!["b", "a"];
    {
        let s = String::from("zz");
        w.sort_by(|a, b| {
            let t: &str = &s;
            t.len().cmp(&a.len()).then(a.cmp(b))
        });
    }
    println!("{:?}", w);
}
