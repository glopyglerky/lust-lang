pub mod b {
    #[derive(Debug)]
    pub struct Deep {
        pub v: i64,
    }
}

// The same short name as m0::Stats, with another field: both print Stats.
#[derive(Debug)]
pub struct Stats {
    pub total: i64,
}
