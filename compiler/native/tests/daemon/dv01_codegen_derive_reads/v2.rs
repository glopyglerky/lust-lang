// Derive edits whose effect only codegen sees: the `#[default]` variant a
// body's `Default::default()` builds, and derived vs written comparisons.
// The bodies below never change; their C must follow the derive lists.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Mode {
    Fast,
    #[default]
    Slow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Rank(u8, u8);

fn start() -> Mode {
    Mode::default()
}

fn before(a: Rank, b: Rank) -> bool {
    a < b
}

fn top(v: &[Rank]) -> Rank {
    let mut m = v[0];
    for &r in v {
        if r > m {
            m = r;
        }
    }
    m
}

fn main() {
    println!("{:?}", start());
    println!("{}", before(Rank(1, 9), Rank(2, 0)));
    println!("{:?}", top(&[Rank(3, 1), Rank(1, 7), Rank(3, 0)]));
}
