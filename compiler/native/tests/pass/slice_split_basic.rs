fn main() {
    let s = b"a,b,,c";
    for p in s.split(|&b| b == b',') { println!("{:?}", p); }
    let v = [1, 2, 0, 3, 0, 0, 4];
    let parts: Vec<&[i32]> = v.split(|x| *x == 0).collect();
    println!("{:?}", parts);
    println!("{:?}", v.rsplit(|x| *x == 0).collect::<Vec<_>>());
    println!("{:?}", v.splitn(2, |x| *x == 0).collect::<Vec<_>>());
    println!("{:?}", v.rsplitn(2, |x| *x == 0).collect::<Vec<_>>());
    println!("{:?}", v.split_inclusive(|x| *x == 0).collect::<Vec<_>>());
    let mut m = [1, 0, 2, 3];
    for g in m.split_mut(|x| *x == 0) { g.reverse(); }
    println!("{:?}", m);
}
