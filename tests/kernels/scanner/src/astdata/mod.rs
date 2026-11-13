//! Kernel mirror of the ts-rust astdata module: only SyntaxKind and the
//! text types, which are all the scanner uses. Both files are verbatim.

mod syntax_kind;
pub mod text;

pub use syntax_kind::SyntaxKind;
// Note: astdata::text::TextRange is a different type from the
// core::TextRange the scanner uses (both exist in ts-rust). It is
// available as crate::astdata::text::TextRange and not re-exported
// here, to keep the prelude name TextRange unambiguous.
