enum Maybe<T> {
    Some(T),
    None,
}

fn main() {
    let m = Maybe::Some(3);
    match m {
        Maybe::Some(v) => println!("{}", v),
        Maybe::None => println!("none"),
    }
}
