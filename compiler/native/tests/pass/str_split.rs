// The str split family (W-STD-B): items, double-endedness, size_hint, and
// capacities as std, for char, &str (also &String and the empty needle),
// char-set, and closure patterns.
fn main() {
    let v: Vec<&str> = "a,b,,c,".split(',').collect();
    println!("{:?} {}", v, v.capacity());
    println!("{:?}", "".split(',').collect::<Vec<_>>());
    println!("{:?}", "a--b----c".split("--").collect::<Vec<_>>());
    println!("{:?}", "abc".split("").collect::<Vec<_>>());
    println!("{:?} {:?}", "aaaa".split("aa").collect::<Vec<_>>(), "aaaa".rsplit("aa").collect::<Vec<_>>());
    println!("{:?}", "a,b,,c,".rsplit(',').collect::<Vec<_>>());
    println!("{:?} {:?}", "a,b,".split_terminator(',').collect::<Vec<_>>(), "a,b,,".split_terminator(',').collect::<Vec<_>>());
    println!("{:?}", "a,b,".rsplit_terminator(',').collect::<Vec<_>>());
    println!("{:?} {:?}", "a,b,c".splitn(2, ',').collect::<Vec<_>>(), "a,b,c".splitn(0, ',').collect::<Vec<_>>());
    println!("{:?}", "a,b,c".rsplitn(2, ',').collect::<Vec<_>>());
    println!("{:?}", "a\nb\n".split_inclusive('\n').collect::<Vec<_>>());
    println!("{:?}", "  a b\t\u{3000}c  ".split_whitespace().collect::<Vec<_>>());
    let w = " a  b ".split_ascii_whitespace();
    println!("{:?}", w.size_hint());
    println!("{:?}", w.collect::<Vec<_>>());
    println!("{:?} {:?}", "a\r\nb\n\nc".lines().collect::<Vec<_>>(), "a\n".lines().collect::<Vec<_>>());
    println!("{:?}", "a1b22c".split(|c: char| c.is_ascii_digit()).collect::<Vec<_>>());
    println!("{:?}", "a,b;c".split([',', ';']).collect::<Vec<_>>());
    println!("{:?}", "héllo wörld".split('ö').collect::<Vec<_>>());
    let mut s = "a,b,c".split(',');
    println!("{:?} {:?} {:?}", s.next_back(), s.next(), s.collect::<Vec<_>>());
    let mut l = "x\ny\nz".lines();
    println!("{:?} {:?}", l.next_back(), l.next());
    println!("{:?} {:?} {:?}", "k=v=w".split_once('='), "k=v=w".rsplit_once('='), "kv".split_once("="));
    println!("{:?}", "a,b".split(',').rev().collect::<Vec<_>>());
    println!("{:?}", " x  y ".split_whitespace().rev().collect::<Vec<_>>());
    let owned = String::from("p q r");
    let parts: Vec<String> = owned.split(' ').map(|x| x.to_uppercase()).collect();
    println!("{:?} {}", parts, owned.split(' ').count());
    let sep = String::from("::");
    println!("{:?}", "a::b".split(&sep).collect::<Vec<_>>());
}
