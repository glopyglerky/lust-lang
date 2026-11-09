enum E {
    V(i64, i64, i64),
}

fn main() {
    let e = E::V(1, 2, 3);
    match e {
        E::V(first, ..) => println!("{}", first),
    }
}
