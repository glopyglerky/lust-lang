// panic: left: Stats { n: 1 }
// An assert_eq panic formats both sides with Debug: source names only.
mod m0 {
    #[derive(Debug, PartialEq)]
    pub struct Stats {
        pub n: i64,
    }
}

fn main() {
    println!("before");
    assert_eq!(m0::Stats { n: 1 }, m0::Stats { n: 2 });
}
