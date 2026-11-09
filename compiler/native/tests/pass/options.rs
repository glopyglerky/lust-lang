// Option<T>, if let, while let, Vec and str helpers, chars().
#[derive(Debug, Clone, PartialEq)]
struct Item {
    name: String,
    qty: u32,
}

fn find(items: &Vec<Item>, name: &str) -> Option<usize> {
    let mut i = 0;
    while i < items.len() {
        if items[i].name == name {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn digits(s: &str) -> u32 {
    let mut total = 0;
    for c in s.chars() {
        if let Some(d) = c.to_digit(10) {
            total += d;
        }
    }
    total
}

fn first_word(s: &str) -> String {
    let mut out = String::new();
    for c in s.trim().chars() {
        if c.is_ascii_whitespace() {
            break;
        }
        out.push(c.to_ascii_uppercase());
    }
    out
}

fn main() {
    let mut items = vec![
        Item { name: "bolt".to_string(), qty: 3 },
        Item { name: "nut".to_string(), qty: 0 },
    ];
    println!("{:?} {:?}", find(&items, "nut"), find(&items, "gear"));
    match find(&items, "bolt") {
        Some(i) => items[i].qty += 10,
        None => println!("missing"),
    }
    let last: Option<&Item> = items.last();
    println!("{:?}", last);
    if let Some(it) = items.first() {
        println!("first {} {}", it.name, it.qty);
    }
    let popped = items.pop();
    println!("{:?} {}", popped, items.len());
    let name = match popped {
        Some(item) => item.name,
        None => String::from("none"),
    };
    println!("{}", name);
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        print!("{} ", top);
    }
    println!();
    let maybe: Option<i64> = None;
    println!("{} {} {}", maybe.unwrap_or(7), maybe.is_none(), Some(5).unwrap());
    println!("{}", digits("a1b22c333"));
    println!("[{}]", first_word("   hello world  "));
    println!("{} {} {}", "hello".contains("ell"), "hello".starts_with("he"), "hello".ends_with("lo"));
    println!("{:?} {}", "  pad  ".trim(), "MiXed".to_lowercase());
    let mut v = vec![10, 20, 30];
    v.insert(1, 15);
    let r = v.remove(0);
    v.swap(0, 2);
    println!("{:?} {} {} {}", v, r, v.contains(&30), v.contains(&99));
    v.truncate(1);
    println!("{:?} {:?}", v, v.get(5));
    let words = vec![Some("a".to_string()), None];
    println!("{:?} {}", words, words[0] == Some("a".to_string()));
    println!("{}", "héllo".chars().count());
}
