// std::cmp::min/max on types that are not builtin scalars call Ord::min/max.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct P(i32, u8);

fn main() {
    let a = P(3, 1);
    println!("{:?} {:?}", std::cmp::min(a, P(1, 9)), std::cmp::max(a, P(3, 2)));
    println!("{:?} {:?}", std::cmp::max(P(1, 1), P(1, 1)), std::cmp::min("b", "a"));
    println!("{:?}", std::cmp::max(String::from("x"), String::from("y")));
}
