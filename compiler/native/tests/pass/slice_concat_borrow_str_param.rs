// `[S]::concat` and `join` over a type parameter `S: Borrow<str>`
// (ts-rust's jsdoc `joined_parts`): each instance takes the String or
// `&str` builtin, with core's exact capacity.
use std::borrow::Borrow;

fn joined_parts<S: Borrow<str>>(parts: &[S]) -> Vec<String> {
    if parts.is_empty() { Vec::new() } else { vec![parts.concat()] }
}

fn dashed<S>(parts: &[S]) -> String
where
    S: Borrow<str>,
{
    parts.join("-")
}

fn main() {
    println!("{:?}", joined_parts(&["a", "bc", ""]));
    let owned = vec![String::from("x"), String::from("yz")];
    let j = joined_parts(&owned);
    println!("{:?} {}", j, j[0].capacity());
    println!("{:?}", joined_parts::<&str>(&[]));
    let d = dashed(&owned);
    println!("{} {}", d, d.capacity());
    println!("{:?}", dashed::<String>(&[]));
}
