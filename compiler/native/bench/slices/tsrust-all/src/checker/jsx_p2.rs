use crate::prelude::*;

// Port of checker/jsx.go lines 929-1482.
//
// PORT: Go `JsxNames.X` fields are constant strings. This file uses the
// string values directly ("JSX", "IntrinsicElements", "ElementClass",
// "ElementAttributesProperty", "ElementChildrenAttribute", "Element",
// "ElementType", "IntrinsicAttributes", "IntrinsicClassAttributes",
// "LibraryManagedAttributes"), so it does not depend on how the first JSX
// file names the `JsxNames` struct.
//
// PORT: `JsxElementLinks` (defined with the first part of jsx.go) is assumed
// to have the Go fields in snake case: `jsx_flags: JsxFlags`,
// `resolved_jsx_element_attributes_type: TypeId`, `jsx_namespace: SymbolId`,
// `jsx_implicit_import_container: SymbolId`.


// Go: checker/jsx.go:1443 markAsSynthetic
// PORT: the isolated parse has no file store, so its nodes are factory
// nodes and `set_node_loc` can write them.
