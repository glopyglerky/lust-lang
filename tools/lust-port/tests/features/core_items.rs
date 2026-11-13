// feature: core.items
// form: const, static, const fn, type alias, tuple struct, enum discriminant, nested fn
const LIMIT: i32 = 10;
static NAME: &str = "n";
const fn dbl(x: i32) -> i32 { x * 2 }
type NodeId = u32;
struct Node(u64);
enum Kind { A = 1, B = 4 }
fn main() {
    fn inner(x: i32) -> i32 { x + 1 }
    let id: NodeId = 3;
    println!("{} {} {} {} {} {}", LIMIT, NAME, dbl(2) + inner(0), id, Node(9).0, Kind::B as i32);
}
