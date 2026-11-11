mod implementation {
    pub fn answer() -> i64 {
        42
    }
}

pub fn exposed() -> i64 {
    implementation::answer()
}
