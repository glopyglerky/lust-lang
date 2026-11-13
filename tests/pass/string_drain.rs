// String::drain: char-wise removal, both ends, as_str, and a
// leaked Drain (the String stays whole, unlike a Vec).

fn main() {
    let mut s = String::from("héllo");
    let d: String = s.drain(1..4).collect();
    println!("{d} | {s}");
    let mut sd = s.drain(..);
    println!("{:?} {:?} {:?} {:?}", sd.next(), sd.next_back(), sd.as_str(), sd.size_hint());
    drop(sd);
    println!("after {s:?}");
    // Early drop removes what was not yielded.
    let mut s = String::from("abcdef");
    {
        let mut d = s.drain(1..5);
        println!("{:?} {:?}", d.next(), d.as_str());
    }
    println!("early {s:?}");
    // Ranges by stored value and by bound pair.
    let r = ..2;
    let x: String = s.drain(r).collect();
    println!("stored {x:?} {s:?}");
    let mut s = String::from("wxyz");
    let x: String = s.drain((std::ops::Bound::Included(1), std::ops::Bound::Unbounded)).collect();
    println!("pair {x:?} {s:?}");
    // A leaked string Drain drops nothing.
    let mut s = String::from("abcd");
    let d = s.drain(1..3);
    std::mem::forget(d);
    println!("forget {s:?}");
    // Empty drains and `.rev()`.
    let mut s = String::from("abc");
    println!("empty {}", s.drain(2..2).count());
    let rev: String = s.drain(..).rev().collect();
    println!("rev {rev:?} {s:?}");
}
