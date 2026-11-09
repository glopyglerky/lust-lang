// A user type named Ordering shadows the flat name only: std::cmp::Ordering
// paths, cmp, and partial_cmp still mean the core enum.
#[derive(Debug, PartialEq)]
enum Ordering { Less, More }
fn f() -> std::cmp::Ordering { std::cmp::Ordering::Less }
fn g(o: std::cmp::Ordering) -> bool { matches!(o, std::cmp::Ordering::Greater) }
fn main() {
    let c: std::cmp::Ordering = 5.cmp(&3);
    println!("{:?} {:?} {} {:?} {:?}", f(), c, g(c), Ordering::Less, Ordering::More);
    println!("{:?} {}", 2u8.partial_cmp(&1), f() == std::cmp::Ordering::Less);
}
