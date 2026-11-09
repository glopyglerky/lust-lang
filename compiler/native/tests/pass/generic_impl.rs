struct Wrapper<T> {
    v: T,
}

impl<T> Wrapper<T> {
    fn new(v: T) -> Wrapper<T> {
        Wrapper { v }
    }
}

fn main() {
    let w = Wrapper::new(3);
    println!("{}", w.v);
}
