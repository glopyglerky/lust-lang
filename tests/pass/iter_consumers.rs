// Iterator consumers (W-STD-A phase 3).
fn main() {
    let v = vec![4, 8, 15, 16, 23, 42];
    println!("{} {:?} {:?}", v.iter().count(), v.iter().last(), v.iter().nth(2));
    println!("{} {}", v.iter().any(|x| *x > 40), v.iter().all(|x| *x > 4));
    println!("{:?} {:?}", v.iter().find(|x| **x > 10), v.iter().position(|x| *x == 16));
    println!("{:?}", v.iter().find_map(|x| if *x > 15 { Some(x * 2) } else { None }));
    println!("{}", v.iter().fold(String::new(), |acc, x| format!("{acc}<{x}>")));
    println!("{:?} {:?}", v.iter().min(), v.iter().max());
    let e: Vec<i32> = Vec::new();
    println!("{:?} {:?} {:?}", e.iter().min(), e.iter().last(), e.iter().find(|_| true));
    let words = vec!["pear", "fig", "kiwi", "date", "yam"];
    println!("{:?} {:?}", words.iter().max_by_key(|w| w.len()), words.iter().min_by_key(|w| w.len()));
    println!("{:?} {:?}", words.iter().max(), words.iter().min());
    let pairs = vec![(2, 'b'), (1, 'z'), (2, 'a')];
    println!("{:?} {:?}", pairs.iter().max(), pairs.iter().min());
    let s: i64 = v.iter().map(|x| *x as i64).sum();
    let p: u64 = (1..=10u64).product();
    let f: f64 = [0.25, 0.5].iter().sum();
    println!("{} {} {}", s, p, f);
    let mut it = v.iter();
    let a = it.next();
    let b = it.next();
    println!("{:?} {:?} {:?} {}", a, b, it.next(), it.count());
    let joined: String = words.iter().copied().collect();
    let shout: String = "quiet".chars().map(|c| c.to_ascii_uppercase()).collect();
    let back: String = vec![String::from("ab"), String::from("cd")].into_iter().collect();
    println!("{} {} {}", joined, shout, back);
    let mut n = 0;
    (0..5).for_each(|i| n += i);
    println!("{}", n);
}
