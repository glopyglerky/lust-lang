struct Wrapper<T> {
    v: T,
}

fn main() {
    let w = Wrapper { v: 3 };
    println!("{}", w.v);
}
