// `partition` and `unzip` fill `Vec<T>` sides from `&T` items when `T: Copy`
// (alloc's `impl Extend<&T> for Vec<T>`), also `&&T` into `Vec<&T>`; owned
// items still fill as before.
fn main() {
    let base: Vec<u32> = (0..10).collect();
    let (ev, od): (Vec<u32>, Vec<u32>) = base.iter().partition(|x| **x % 2 == 0);
    println!("{:?} {:?}", ev, od);
    let (lo, hi): (Vec<u32>, Vec<u32>) = base.iter().partition(|x| **x < 3);
    println!("{:?} {}", lo, hi.len());
    let pairs = vec![(1u8, 'a'), (2, 'b'), (3, 'c')];
    let (ns, cs): (Vec<u8>, Vec<char>) = pairs.iter().map(|(n, c)| (n, c)).unzip();
    println!("{:?} {:?}", ns, cs);
    let words = vec!["x", "yy", "zzz"];
    let (short, long): (Vec<&str>, Vec<&str>) = words.iter().partition(|w| w.len() < 2);
    println!("{:?} {:?}", short, long);
    let refs: Vec<&u32> = base.iter().collect();
    let (a, b): (Vec<&u32>, Vec<&u32>) = refs.iter().partition(|r| ***r > 7);
    println!("{:?} {:?}", a, b);
    let (o1, o2): (Vec<String>, Vec<String>) = vec!["p".to_string(), "qq".to_string()].into_iter().partition(|s| s.len() == 1);
    println!("{:?} {:?}", o1, o2);
}
