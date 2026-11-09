// A const fn edit moves SEED's value with no change to the closure's or the
// generic instance's source: both read SEED (a Sig read folds its value) and
// must check again. v2 also adds a const in another fn's body, so the request
// takes a full resolve, not the body patch (fuzz small2 seed 11, 2026-10-19:
// the warm closure kept the old value).
const fn seed_val() -> i64 {
    4
}

const SEED: i64 = seed_val();

fn label(n: i64) -> i64 {
    n
}

fn use_closure(n: i64) -> i64 {
    let add_seed = |x: i64| x + SEED;
    add_seed(n)
}

fn use_generic<T: Into<i64>>(n: T) -> i64 {
    n.into() + SEED
}

fn main() {
    println!("{} {} {}", label(1), use_closure(10), use_generic(100i64));
}
