// In-place collect rules per adapter, the direct vec::IntoIter form
// (reuse unless advanced past half), and the byte-size capacity rule.
fn mk() -> Vec<u64> {
    let mut v: Vec<u64> = Vec::with_capacity(10);
    v.extend([1, 2, 3, 4, 5]);
    v
}
fn mk2() -> Vec<[u64; 2]> {
    let mut v: Vec<[u64; 2]> = Vec::with_capacity(10);
    v.extend([[1, 1], [2, 2], [3, 3]]);
    v
}
fn main() {
    let other = vec![7u64; 9];
    let en: Vec<(usize, u64)> = mk2().into_iter().map(|a| a[0]).enumerate().collect();
    let zi: Vec<(u64, u64)> = mk2().into_iter().map(|a| a[0]).zip(other.iter().copied()).collect();
    let pk: Vec<u64> = mk().into_iter().peekable().collect();
    let fu: Vec<u64> = mk().into_iter().take(9).collect();
    let sc: Vec<u64> = mk().into_iter().skip(0).collect();
    let mw: Vec<u64> = mk().into_iter().take_while(|x| *x < 4).collect();
    let fl: Vec<u64> = mk2().into_iter().flatten().collect();
    let fm: Vec<u64> = mk().into_iter().flat_map(|x| [x, x]).collect();
    let direct: Vec<u64> = mk().into_iter().collect();
    let mut it = mk().into_iter();
    it.next();
    let adv: Vec<u64> = it.collect();
    let mut it2 = mk().into_iter();
    it2.next();
    it2.next();
    it2.next();
    let adv2: Vec<u64> = it2.collect();
    let mut it3 = mk().into_iter();
    it3.next();
    let adv3: Vec<u64> = it3.map(|x| x * 2).collect();
    let two: Vec<u32> = vec![1u64, 2, 3].into_iter().map(|x| x as u32).collect();
    let pairs: Vec<[u32; 2]> = mk().into_iter().map(|x| [x as u32, 0]).collect();
    let halves: Vec<u32> = mk2().into_iter().map(|a| a[0] as u32).collect();
    let up: Vec<[u64; 2]> = mk().into_iter().map(|x| [x, x]).collect();
    let mut odd: Vec<[u8; 3]> = Vec::with_capacity(5);
    odd.extend([[1, 2, 3], [4, 5, 6]]);
    let odd2: Vec<[u8; 2]> = odd.into_iter().map(|a| [a[0], a[1]]).collect();
    println!("en {} zi {} pk {} fu {} sc {} mw {} fl {} fm {}", en.capacity(), zi.capacity(), pk.capacity(), fu.capacity(), sc.capacity(), mw.capacity(), fl.capacity(), fm.capacity());
    println!("direct {} adv {} {:?} adv2 {} {:?} adv3 {} {:?}", direct.capacity(), adv.capacity(), adv, adv2.capacity(), adv2, adv3.capacity(), adv3);
    println!("two {} pairs {} halves {} up {} odd2 {}", two.capacity(), pairs.capacity(), halves.capacity(), up.capacity(), odd2.capacity());
}
