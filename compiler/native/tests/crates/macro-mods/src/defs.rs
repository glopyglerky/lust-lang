macro_rules! twice {
    ($e:expr) => {
        $e + $e
    };
}

pub fn base() -> i64 {
    twice!(10)
}
