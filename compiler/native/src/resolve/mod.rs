// Name resolution: the crate graph, the module tree, the import worklist,
// cfg stripping, and the outline worklist driver (docs/ARCH.md sections 1
// and 6.1). W-RESOLVE owns this directory and src/scan.rs.
pub mod adapters;
pub mod cargo;
pub mod cfg;
pub mod imports;
pub mod macro_scope;
pub mod mdr;
pub mod modules;
pub mod order;
pub mod prefetch;
pub mod probe;
pub mod stdpaths;
pub mod worklist;
