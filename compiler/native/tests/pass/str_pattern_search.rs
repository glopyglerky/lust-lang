// The str search methods over every `Pattern` kind lust takes
// (tf-str): char, `&str`, `&String`, char arrays and slices, and
// `FnMut(char) -> bool` closures, with empty and multi-byte cases.
use std::cell::RefCell;

fn main() {
    let s = "héllo-wörld";
    println!("{:?} {:?}", s.find(['-', 'ö']), s.rfind(['-', 'ö']));
    println!("{:?} {:?}", s.find(&['x', 'l'][..]), s.rfind(&"llo"));
    let set = ['w', 'ö'];
    println!("{:?} {}", s.find(&set), s.contains(['z', 'h']));
    println!("{:?}", s.find(|c: char| c.is_numeric()));
    println!("{:?} {:?}", "ab1cd2".find(|c: char| c.is_ascii_digit()), "ab1cd2".rfind(|c: char| c.is_ascii_digit()));
    println!("{} {}", s.starts_with(['h', 'x']), s.ends_with(['d', 'x']));
    println!("{} {}", s.starts_with(|c: char| c == 'h'), "".starts_with('a'));
    println!("{:?} {:?}", s.strip_prefix(['h']), s.strip_suffix(|c: char| c == 'd'));
    println!("{:?} {:?}", "abc".strip_prefix(""), "abc".strip_suffix(['x', 'c']));
    let owned = String::from("-x-");
    println!("{:?} {:?} {}", owned.find(&String::from("x")), owned.rfind('-'), owned.contains(['-']));
    // Predicate calls run once per char in scan order, and stop at
    // the first match (backwards for `rfind`).
    let log = RefCell::new(String::new());
    let r = "ab1cd".find(|c: char| {
        log.borrow_mut().push(c);
        c.is_ascii_digit()
    });
    println!("{:?} {}", r, log.borrow());
    let log2 = RefCell::new(String::new());
    let r = "ab1c2".rfind(|c: char| {
        log2.borrow_mut().push(c);
        c.is_ascii_digit()
    });
    println!("{:?} {}", r, log2.borrow());
    let log3 = RefCell::new(String::new());
    let r = "abc".starts_with(|c: char| {
        log3.borrow_mut().push(c);
        c == 'x'
    });
    println!("{} {}", r, log3.borrow());
    // Closures by value (FnMut with a counter), and empty patterns.
    let mut n = 0;
    let every3 = move |_: char| {
        n += 1;
        n % 3 == 0
    };
    println!("{:?}", "abcdef".find(every3));
    println!("{:?} {:?} {:?} {:?}", "abc".find(""), "abc".rfind(""), "".find(['a']), "abc".find(['x'; 0]));
    let empty: &[char] = &[];
    println!("{:?} {}", "abc".find(empty), "abc".contains(empty));
}
