pub struct Wrap(pub i64, i64);

fn main() {
    let w = Wrap(3, 4);
    println!("{}", w.0);
}
