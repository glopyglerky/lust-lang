// expect: L9001
fn main() {
    let s = ["a".to_string(), "b".to_string()];
    match s {
        [first, rest @ .., last] if rest.len() > 5 => println!("{} {}", first, last),
        [a, b] => println!("{} {}", a, b),
        _ => println!("no"),
    }
}
