// String building, comparison across String and &str, escapes in Debug output.
fn greet(name: &str) -> String {
    let mut s = String::from("hello, ");
    s.push_str(name);
    s.push('!');
    s
}

fn longest(words: &Vec<String>) -> String {
    let mut best = String::new();
    for w in words {
        if w.len() > best.len() {
            best = w.clone();
        }
    }
    best
}

fn main() {
    let g = greet("world");
    println!("{}", g);
    println!("{:?}", g);
    let words = vec!["apple".to_string(), "fig".to_string(), "banana".to_string()];
    println!("{}", longest(&words));
    let a = String::from("abc");
    let b = "abd";
    println!("{} {} {}", a == "abc", a.as_str() < b, b > "abc");
    let c = a.clone() + "def" + &g;
    println!("{} {}", c, c.len());
    let mut t = String::new();
    t += "x";
    t += &c;
    println!("{} {}", t.is_empty(), t);
    println!("{:?} {:?}", "tab\there \"quoted\" back\\slash", 'q');
    let e = "héllo wörld";
    println!("{} {} {:?}", e, e.len(), e);
}
