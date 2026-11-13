fn main() {
    let f = |(x, y): (i64, i64)| x + y;
    println!("{}", f((1, 2)));
}
