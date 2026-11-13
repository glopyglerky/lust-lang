// The second rank of String methods, all of it around the
// char-boundary rules: pop, remove, insert and insert_str,
// truncate, clear, split_off, as_bytes, into_bytes, and the
// capacity family with Rust's growth numbers.
fn main() {
    let mut s = "héllo".to_string();
    println!("{} {:?}", s.capacity(), s.pop());
    println!("{:?} {}", s.remove(1), s);
    s.insert(1, 'é');
    s.insert_str(0, ">>");
    println!("{}", s);
    let tail = s.split_off(3);
    println!("{:?} {:?} {}", s, tail, tail.capacity());
    let mut t = "abcdef".to_string();
    t.truncate(3);
    println!("{}", t);
    t.clear();
    println!("{} {}", t.is_empty(), t.capacity());
    println!("{:?}", "hello".to_string().as_bytes());
    let b = "hi".to_string().into_bytes();
    println!("{:?} {}", b, b.capacity());
    let mut c = String::new();
    println!("{}", c.capacity());
    c.push_str("hello");
    println!("{}", c.capacity());
    c.push_str(" world, this is longer");
    println!("{} {}", c.capacity(), c.len());
    c.reserve(100);
    println!("{}", c.capacity());
    c.reserve_exact(5);
    println!("{}", c.capacity());
    c.shrink_to(40);
    println!("{}", c.capacity());
    c.shrink_to_fit();
    println!("{}", c.capacity());
    let ch = String::from('é');
    println!("{} {}", ch, ch.capacity());
    let owned = String::from("owned".to_string());
    println!("{} {}", owned, owned.capacity());
    let mut e = String::new();
    println!("{:?}", e.pop());
    e.insert(0, 'x');
    e.insert_str(0, "abc");
    println!("{} {}", e, e.capacity());
    let mut m = "middle".to_string();
    m.insert_str(3, "XXX");
    m.truncate(30);
    println!("{}", m);
}
