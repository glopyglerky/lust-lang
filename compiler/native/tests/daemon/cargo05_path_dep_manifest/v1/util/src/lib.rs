pub fn f() -> &'static str {
    if cfg!(feature = "x") { "x" } else { "plain" }
}
#[cfg(feature = "y")]
pub fn g() -> u32 { "no" }
