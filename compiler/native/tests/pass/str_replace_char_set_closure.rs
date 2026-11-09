// str::replace and replacen over closures, fn items, and char arrays and slices: alloc's match_indices loop, capacity 0 (replace) or 32 (replacen), the predicate called only up to the last replaced match.
fn main() { more();
    let s = "a-b_c-d";
    println!("{}", s.replace('-', "+"));
    println!("{}", s.replace("-", "+"));
    println!("{}", s.replace(|c: char| c == '_' || c == '-', "+"));
    println!("{}", s.replace(&['-', '_'][..], "+"));
    println!("{}", s.replace(['-', '_'], "+"));
    println!("{}", s.replacen('-', "+", 1));
    println!("{}", s.replacen(|c: char| c == '_' || c == '-', "+", 2));
    println!("{}", s.replacen(&['-', '_'][..], "+", 2));
    println!("{}", s.replacen(['-', '_'], "+", 1));
    println!("{}", s.replacen(char::is_alphabetic, "X", 3));
}
fn more() {
    let s = String::from("héllo wörld");
    let mut seen = Vec::new();
    let r = s.replacen(|c: char| { seen.push(c); !c.is_ascii() }, "<>", 1);
    println!("{} {} {:?}", r, r.capacity(), seen);
    let r2 = s.replace(&['l', 'o'], "");
    println!("{} {}", r2, r2.capacity());
    let r3 = "aaa".replacen(['a'], "bb", 0);
    println!("{} {}", r3, r3.capacity());
    let to = String::from("--");
    let r4 = s.as_str().replace(char::is_whitespace, &to);
    println!("{} {}", r4, r4.capacity());
    let arr = ['x'];
    println!("{}", "xyx".replacen(&arr, "Z", 5));
}
