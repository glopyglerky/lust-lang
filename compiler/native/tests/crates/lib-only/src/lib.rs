// A lib target has no main; lust check --lib accepts it as rustc does.
pub fn double(x: u32) -> u32 {
    x * 2
}

pub struct Pair {
    pub a: u32,
}
