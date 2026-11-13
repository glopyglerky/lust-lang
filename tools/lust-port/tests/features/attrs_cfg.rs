// feature: attrs.cfg
// form: cfg and cfg! on the target, inline, must_use and allow
#[inline]
#[must_use]
fn pick() -> i32 { if cfg!(unix) { 1 } else { 2 } }
#[cfg(target_os = "linux")]
fn os() -> i32 { 10 }
#[cfg(not(target_os = "linux"))]
fn os() -> i32 { 20 }
#[allow(dead_code)]
fn unused() {}
fn main() { println!("{} {}", pick(), os()); }
