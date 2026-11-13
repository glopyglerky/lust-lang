// capability: rejected: match guards over a by-value slice-pattern middle
// expect: L9001

fn main() {
    let v = [String::new(), String::new(), String::new()];
    match v {
        [a, mid @ .., b] if mid.len() == 1 => println!("{} {} {}", a, mid.len(), b),
        _ => {}
    }
}
