// std::cmp::max on String calls Ord::max.
fn main() {
    println!("{}", std::cmp::max(String::from("a"), String::from("b")));
}
