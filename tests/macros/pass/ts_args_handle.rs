// macro: args, handle, arena_link_key
// source: crates/ts_goport/src/core.rs:3836
// source: crates/ts_goport/src/core.rs:18
// source: crates/ts_goport/src/core.rs:3868
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
// Minimal stand-ins for the traits arena_link_key! writes impls for.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LinkSlot { Arena(usize) }
pub trait LinkKey { fn link_slot(self) -> LinkSlot; }

macro_rules! handle {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
        pub struct $name(pub u32);

        impl $name {
            /// Go `nil`.
            pub const NIL: Self = Self(0);

            #[must_use]
            pub const fn is_nil(self) -> bool {
                self.0 == 0
            }

            #[must_use]
            pub const fn is_some(self) -> bool {
                self.0 != 0
            }

            /// Arena index. Arenas keep a dummy entry at index 0.
            #[must_use]
            pub const fn index(self) -> usize {
                self.0 as usize
            }

            /// Returns `None` for nil.
            #[must_use]
            pub const fn get(self) -> Option<Self> {
                if self.0 == 0 { None } else { Some(self) }
            }
        }
    };
}

macro_rules! arena_link_key {
    ($($name:ident),*) => {$(
        impl LinkKey for $name {
            #[inline]
            fn link_slot(self) -> LinkSlot {
                LinkSlot::Arena(self.index())
            }
        }
    )*};
}

#[macro_export]
macro_rules! args {
    ($($arg:expr),* $(,)?) => { vec![$(::std::string::ToString::to_string(&$arg)),*] };
}

handle!(NodeId);
handle!(
    /// A documented handle, as ts-rust writes them.
    SymbolId
);
arena_link_key!(NodeId, SymbolId);

fn main() {
    let nil = NodeId::NIL;
    let some = NodeId(7);
    println!("nil={} some={} index={} get={:?}", nil.is_nil(), some.is_some(), some.index(), some.get());
    println!("slot={:?} ord={}", some.link_slot(), NodeId(3) < NodeId(7));
    let words: Vec<String> = args!["alpha", 42, true];
    println!("args={words:?} empty={:?}", args![] as Vec<String>);
    // The path form of a #[macro_export] macro, from a child module.
    println!("path={:?}", crate::args!["x"]);
}
