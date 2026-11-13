// `cmp` on strings, `Ordering::{reverse, then, then_with, is_*}`, in
// comparator closures (W-STD-A phase 3).
use std::cmp::Ordering;
struct P {
    name: String,
    age: u32,
}
fn main() {
    let mut people = vec![
        P { name: String::from("bo"), age: 30 },
        P { name: String::from("al"), age: 25 },
        P { name: String::from("cy"), age: 30 },
        P { name: String::from("al"), age: 40 },
    ];
    people.sort_by(|a, b| b.age.cmp(&a.age).then_with(|| a.name.cmp(&b.name)));
    for p in &people {
        print!("{}:{} ", p.name, p.age);
    }
    println!();
    people.sort_by(|a, b| a.name.cmp(&b.name).then(a.age.cmp(&b.age)).reverse());
    for p in &people {
        print!("{}:{} ", p.name, p.age);
    }
    println!();
    let o = 3.cmp(&5);
    println!("{:?} {:?} {} {} {} {} {} {}", o, o.reverse(), o.is_lt(), o.is_le(), o.is_gt(), o.is_ge(), o.is_eq(), o.is_ne());
    println!("{:?} {:?}", Ordering::Equal.then(Ordering::Greater), "b".cmp("abc"));
    let mut calls = 0;
    let r = Ordering::Less.then_with(|| {
        calls += 1;
        Ordering::Greater
    });
    println!("{:?} {}", r, calls);
    let s = String::from("x");
    println!("{} {} {:?}", s.lt(&String::from("y")), "q".ge("q"), s.partial_cmp(&String::from("w")));
}
