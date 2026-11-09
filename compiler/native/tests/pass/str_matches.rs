// `matches`, `rmatches`, `match_indices`, and `rmatch_indices`
// (tf-str): order, byte indices, empty and multi-byte patterns,
// and double-ended use over char patterns.
fn main() {
    println!("{:?}", "a.b.c".matches('.').collect::<Vec<_>>());
    println!("{:?}", "a.b.c".rmatches('.').collect::<Vec<_>>());
    println!("{:?}", "a.b.c".match_indices('.').collect::<Vec<_>>());
    println!("{:?}", "a.b.c".rmatch_indices('.').collect::<Vec<_>>());
    println!("{:?}", "héllo".match_indices('l').collect::<Vec<_>>());
    println!("{:?}", "ab".matches("").collect::<Vec<_>>());
    println!("{:?}", "ab".match_indices("").collect::<Vec<_>>());
    println!("{:?}", "".matches("").collect::<Vec<_>>());
    println!("{:?}", "aaa".matches("aa").collect::<Vec<_>>());
    println!("{:?}", "aaa".rmatches("aa").collect::<Vec<_>>());
    println!("{:?}", "a1b2c".matches(|c: char| c.is_ascii_digit()).collect::<Vec<_>>());
    println!("{:?}", "a1b2c".match_indices(['1', '2']).collect::<Vec<_>>());
    println!("{}", "one two one".matches("one").count());
    println!("{:?}", "a.b.c".matches('.').size_hint());
    let mut it = "a.b.c".matches('.');
    println!("{:?} {:?} {:?}", it.next(), it.next_back(), it.next());
    let mut it = "a.b.c".match_indices('.');
    println!("{:?} {:?} {:?}", it.next_back(), it.next(), it.next());
    let mut it = "xyx".rmatches(['x', 'y']);
    println!("{:?} {:?} {:?}", it.next(), it.next_back(), it.next());
    let s = String::from("s,s,s");
    println!("{:?}", s.matches(',').collect::<Vec<_>>());
    for (i, m) in "é.é".match_indices('.') {
        println!("{} {}", i, m);
    }
}
