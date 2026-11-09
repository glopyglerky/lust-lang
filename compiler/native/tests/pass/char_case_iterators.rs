// `char::to_lowercase` and `char::to_uppercase` (TRIAGE T8): the
// `core::char` source iterators over the runtime case tables. One
// to three chars, both ends, exact size, and std's Debug and
// Display text of the chars left.
fn read<T>(v: T) -> T {
    v
}

fn main() {
    println!("{}", read('A').to_lowercase().collect::<String>());
    println!("{}", read('ß').to_uppercase().collect::<String>());
    println!("{}", read('İ').to_lowercase().collect::<String>());
    println!("{}", read('Σ').to_lowercase().collect::<String>());
    println!("{}", read('ς').to_uppercase().collect::<String>());
    println!("{}", read('ﬀ').to_uppercase().collect::<String>());
    let mut it = read('İ').to_lowercase();
    println!("{:?} {:?} {:?}", it.next(), it.next(), it.next());
    println!("{:?}", read('ﬀ').to_uppercase().next_back());
    let mut it2 = read('ﬀ').to_uppercase();
    println!("{:?} {:?} {:?}", it2.next_back(), it2.next(), it2.next_back());
    println!("{:?}", read('A').to_lowercase());
    println!("{:?}", read('ß').to_uppercase());
    let mut it3 = read('İ').to_lowercase();
    it3.next();
    println!("{:?} {:?} {}", it3, it3.size_hint(), it3.len());
    println!("{}", read('ß').to_uppercase());
    println!("{}", read('ǅ').to_lowercase().count());
    let v: Vec<char> = read('İ').to_lowercase().collect();
    println!("{:?}", v);
    for c in read('K').to_lowercase() {
        println!("loop {c}");
    }
    let first = |c: char| c.to_lowercase().next().unwrap_or(c);
    println!("{} {} {}", first('A'), first('İ'), first('5'));
    let mut u = read('ǆ').to_uppercase();
    let a = u.next().unwrap_or('ǆ');
    let second = u.next().is_some();
    println!("{a} {second}");
    let cloned: String = read('ﬀ').to_uppercase().clone().collect();
    println!("{cloned}");
    println!("{:?}", read('x').to_lowercase().map(|c| c as u32).collect::<Vec<u32>>());
}
