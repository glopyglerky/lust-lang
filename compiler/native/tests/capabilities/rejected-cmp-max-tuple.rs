// capability: rejected: `cmp::max`/`cmp::min` on tuples (`Ord::max` as a function)
// expect: L0641

fn main() {
    println!("{:?}", std::cmp::max((1, 2), (1, 3)));
}
