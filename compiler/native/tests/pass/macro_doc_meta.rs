// A doc comment in a macro call is `#[doc = "..."]`: it matches a
// `meta` fragment inside `#[$m:meta]`, as a written `#[doc = "..."]`
// does. Line docs, block docs, several docs, docs mixed with plain
// attributes, and a doc forwarded through a second macro all count.
macro_rules! handle {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug)]
        pub struct $name(pub u32);
    };
}

macro_rules! fwd {
    ($(#[$meta:meta])* $name:ident) => {
        handle!($(#[$meta])* $name);
    };
}

handle!(
    /// A doc comment in the call.
    TypeId
);

handle!(
    /** A block doc comment. */
    /// And a line doc after it.
    #[doc = "written out"]
    SymbolId
);

fwd!(
    /// Forwarded through one more macro.
    NodeId
);

fn main() {
    println!("{:?} {:?} {:?}", TypeId(3), SymbolId(4), NodeId(5));
}
