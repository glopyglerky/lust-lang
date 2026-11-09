// expect: L0806
// A doc comment is an attribute, not an identifier: with no
// `#[$m:meta]` in the matcher the call has no matching rule.
macro_rules! handle {
    ($name:ident) => {
        pub struct $name(pub u32);
    };
}

handle!(
    /// A doc comment in the call.
    TypeId
);

fn main() {}
