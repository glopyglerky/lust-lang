fn secret() -> i64 {
    9
}

pub(crate) fn crate_only() -> i64 {
    secret()
}

pub fn open() -> i64 {
    crate_only()
}
