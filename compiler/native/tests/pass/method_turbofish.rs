// `str::parse` with a turbofish target (tf-str); was a reject
// case while `parse` was unimplemented.
fn main() {
    let n: i64 = "42".parse::<i64>().unwrap();
    println!("{}", n);
}
