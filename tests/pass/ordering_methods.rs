// core::cmp::Ordering methods: is_eq and kin, reverse, then, max/min,
// and the discriminant cast.
use std::cmp::Ordering;
fn table() {
    let v = [Ordering::Less, Ordering::Equal, Ordering::Greater];
    for a in v { for b in v { print!("{:?}/{:?} ", a.then(b), a.reverse()); } println!(); }
    for a in &v { print!("{}{}{}{}{}{} ", a.is_eq() as u8, a.is_ne() as u8, a.is_lt() as u8, a.is_gt() as u8, a.is_le() as u8, a.is_ge() as u8); }
    println!();
    let r = &v[0];
    println!("{:?} {}", r.reverse(), r.is_lt());
    println!("{:?}", 1.cmp(&2).then(3.cmp(&1)).reverse());
}

fn basics() {
    let a = Ordering::Less;
    println!("{:?} {:?} {:?}", a.reverse(), Ordering::Equal.then(Ordering::Greater), a.then(Ordering::Greater));
    println!("{} {} {} {} {} {}", a.is_lt(), a.is_le(), a.is_gt(), a.is_ge(), a.is_eq(), a.is_ne());

    println!("{:?} {:?}", Ordering::Less.max(Ordering::Greater), a.min(Ordering::Equal));
    println!("{}", a as i8);
}
fn main() {
    table();
    basics();
}
