fn main() {
    let v = vec![1, 2, 3];
    match &v[..] {
        [first, ..] => println!("{}", first),
        [] => println!("empty"),
    }
}
