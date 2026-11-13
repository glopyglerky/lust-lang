// Rust 1.99's Zip: two full TrustedRandomAccess sides read by index
// (no extra pull past the short side); with array::IntoIter (only the
// NoCoerce form, Copy items) `next` is the general one and `fold`
// reads by index. Skip/StepBy over array::IntoIter are not TrustedLen.
fn main() {
    let s7 = [7, 8];
    let s3 = [1, 2, 3];
    for (a, b) in [1, 2, 3, 4].into_iter().map(|x| { println!("e{x}"); x }).zip(s7.iter()) {
        println!("{a}{b}");
    }
    let mut it = (0..5).map(|x| { println!("f{x}"); x }).enumerate().zip(s7.iter().map(|x| { println!("g{x}"); x }));
    while let Some(p) = it.next() {
        println!("{:?}", p);
    }
    println!("{:?} {:?}", it.next(), it.next());
    let n = s3.iter().map(|x| { println!("h{x}"); x }).zip(s7.iter()).count();
    let v: Vec<_> = [1, 2, 3].into_iter().map(|x| { println!("i{x}"); x }).zip(s7.iter()).collect();
    println!("{n} {:?}", v);
    let mut d = [1, 2, 3].into_iter().map(|x| { println!("d{x}"); x }).zip(s7[..1].iter().copied());
    d.next();
    d.next();
    println!("{:?}", d.size_hint());
    let mut it = [String::new(), String::new()].into_iter().skip(1);
    it.next();
    let v: Vec<String> = it.collect();
    let mut it = [1u8, 2, 3, 4].into_iter().step_by(2);
    it.next();
    let w: Vec<u8> = it.collect();
    let mut it = [25i64, 41, 1].into_iter().skip(1);
    it.next();
    let x: Vec<i64> = it.collect();
    let mut it = (0..9u64).skip(1);
    it.next();
    let y: Vec<u64> = it.collect();
    println!("{} {} {} {}", v.capacity(), w.capacity(), x.capacity(), y.capacity());
}
