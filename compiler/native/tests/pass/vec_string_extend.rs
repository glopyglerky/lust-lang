// Vec::extend and String::extend with std's exact capacities: arrays by
// value, a Vec, &Vec (Copy items), a filtered iterator (reserve lower + 1
// when full), repeat_n, a mapped range, chars; String from chars, &str,
// String, and &char items. Also array into_iter chains and repeat_n.
use std::iter::repeat_n;

fn main() {
    let mut v: Vec<u32> = Vec::new();
    v.extend([1u32, 2, 3]);
    println!("{:?} {}", v, v.capacity());
    v.extend(vec![4u32, 5]);
    println!("{:?} {}", v, v.capacity());
    let more = vec![6u32, 7, 8, 9];
    v.extend(&more);
    println!("{:?} {}", v, v.capacity());
    v.extend(more.iter().filter(|x| **x % 2 == 0));
    println!("{:?} {}", v, v.capacity());
    v.extend(repeat_n(0u32, 6));
    println!("{:?} {}", v, v.capacity());
    v.extend((10..13).map(|x| x * 2));
    println!("{:?} {}", v, v.capacity());
    let mut w: Vec<char> = Vec::with_capacity(1);
    w.extend("héllo".chars());
    println!("{:?} {}", w, w.capacity());
    let mut s = String::new();
    s.extend(['a', 'b', 'c']);
    println!("{} {}", s, s.capacity());
    s.extend(repeat_n('z', 10));
    println!("{} {}", s, s.capacity());
    s.extend(vec!["x", "yy"]);
    println!("{} {}", s, s.capacity());
    s.extend(vec![String::from("Q"), String::from("RR")]);
    println!("{} {}", s, s.capacity());
    let cs = ['m', 'n'];
    s.extend(cs.iter());
    println!("{} {}", s, s.capacity());
    let total: u32 = [5u32, 6, 7].into_iter().rev().map(|x| x + 1).sum();
    let pairs: Vec<(u8, char)> = [1u8, 2, 3].into_iter().zip(['a', 'b']).collect();
    println!("{} {:?} {}", total, pairs, pairs.capacity());
    for x in [9, 8].into_iter().enumerate() {
        println!("{:?}", x);
    }
    let rep: Vec<&str> = repeat_n("ab", 3).collect();
    println!("{:?} {}", rep, rep.capacity());
}
