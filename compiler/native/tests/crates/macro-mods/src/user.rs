macro_rules! plus_two {
    ($e:expr) => {
        $e + 2
    };
}

pub fn run() -> i64 {
    plus_two!(crate::defs::base())
}
