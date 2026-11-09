trait Tr {
    const C: u32;
}
struct K;
impl Tr for K {
    const C: u32 = 7;
}
impl K {
    const A: u32 = 5;
}
fn main() {
    match 6u32 {
        <K>::A..=<K as Tr>::C => println!("in"),
        _ => println!("o"),
    }
    match 9u32 {
        <K>::A..=<K as Tr>::C => println!("in"),
        _ => println!("o"),
    }
    match 3u32 {
        0..<K>::A => println!("in"),
        _ => println!("o"),
    }
    match 9u32 {
        ..<K>::A => println!("in"),
        _ => println!("o"),
    }
    match 100u32 {
        <K as Tr>::C.. => println!("in"),
        _ => println!("o"),
    }
    match 4u32 {
        ..=<K>::A => println!("in"),
        _ => println!("o"),
    }
}
