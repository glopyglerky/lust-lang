// expect: L0001
// rustc: visibility `pub` is not followed by an item (in a cfg-removed
// module, a `pub default` is no tuple field).
#[cfg(any())]
mod m {
    pub default
}
fn main() {}
