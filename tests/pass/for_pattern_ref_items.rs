// A `for` pattern that matches the item through references (default
// binding modes) binds into the iterator's item, a reference here: the
// item local has that type (it had the pattern's, a codegen panic).
struct P {
    x: u8,
    name: String,
}

fn lookup<V: Clone>(map: &Vec<(String, V)>, path: &str) -> Option<V> {
    for (k, v) in map {
        if k == path {
            return Some(v.clone());
        }
    }
    None
}

fn main() {
    let v = vec![(1u8, 2u8), (3, 4)];
    for (a, b) in &v {
        println!("{} {}", a, b);
    }
    for (a, b) in v.iter() {
        println!("iter {}", *a + *b);
    }
    let mut w = vec![(1u8, 2u8)];
    for (a, b) in &mut w {
        *a += *b;
    }
    println!("{:?}", w);
    for (a, b) in w.iter_mut() {
        *b = *a * 2;
    }
    println!("{:?}", w);
    let arr = [(String::from("s"), 5u8)];
    for (s, n) in &arr {
        println!("{} {}", s, n);
    }
    let ps = vec![P { x: 7, name: String::from("p") }];
    for P { x, name } in &ps {
        println!("{} {}", x, name);
    }
    let named = vec![(String::from("a"), true)];
    println!("{:?} {:?}", lookup(&named, "a"), lookup(&named, "b"));
}
