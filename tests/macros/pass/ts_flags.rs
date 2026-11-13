// macro: go_flags, go_enum
// source: crates/ts_goport/src/flags_macros.rs:10
// source: crates/ts_goport/src/flags_macros.rs:95
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
#[macro_export]
macro_rules! go_flags {
    ($name:ident, $repr:ty { $($(#[$meta:meta])* $konst:ident = $value:expr;)* }) => {
        #[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub $repr);

        #[allow(non_upper_case_globals)]
        impl $name {
            $($(#[$meta])* pub const $konst: Self = Self($value as $repr);)*

            #[must_use]
            pub const fn bits(self) -> $repr {
                self.0
            }

            /// Go `x&y != 0`.
            #[must_use]
            pub const fn intersects(self, other: Self) -> bool {
                self.0 & other.0 != 0
            }

            /// Go `x&y == y`.
            #[must_use]
            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }

            #[must_use]
            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }

            /// Go `x &^ y`.
            #[must_use]
            pub const fn without(self, other: Self) -> Self {
                Self(self.0 & !other.0)
            }

            #[must_use]
            pub const fn union(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }

        impl std::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }

        impl std::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl std::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }

        impl std::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }

        impl std::ops::Not for $name {
            type Output = Self;
            fn not(self) -> Self {
                Self(!self.0)
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({:#x})", stringify!($name), self.0)
            }
        }
    };
}

#[macro_export]
macro_rules! go_enum {
    ($name:ident, $repr:ty { $($(#[$meta:meta])* $konst:ident = $value:expr;)* }) => {
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub $repr);

        #[allow(non_upper_case_globals)]
        impl $name {
            $($(#[$meta])* pub const $konst: Self = Self($value as $repr);)*
        }
    };
}

go_flags!(NodeFlags, u32 {
    NONE = 0;
    HAS_NAME = 1;
    /// A documented flag.
    HAS_TYPE = 2;
    BOTH = 3;
});
go_enum!(SyntaxKind, u16 {
    UNKNOWN = 0;
    IDENTIFIER = 79;
    FUNCTION = 260;
});

fn main() {
    let f = NodeFlags::HAS_NAME | NodeFlags::HAS_TYPE;
    println!("bits={} both={} name={} empty={} debug={:?}",
        f.bits(), f.contains(NodeFlags::BOTH), f.intersects(NodeFlags::HAS_NAME),
        NodeFlags::NONE.is_empty(), f);
    let g = f.without(NodeFlags::HAS_NAME);
    println!("without={} union={} not_empty={}", g.bits(), g.union(NodeFlags::HAS_NAME).bits(), (!NodeFlags::NONE).bits() != 0);
    let k = SyntaxKind::IDENTIFIER;
    println!("kind={} default={} eq={}", k.0, SyntaxKind::default().0, k == SyntaxKind::IDENTIFIER);
}
