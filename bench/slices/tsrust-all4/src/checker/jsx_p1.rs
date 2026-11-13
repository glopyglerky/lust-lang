//! Port of `checker/jsx.go` lines 1-928.

use crate::jsnum::Number;
use crate::prelude::*;

// PORT: Go `JsxFlags` and `JsxReferenceKind` (jsx.go:17-32) are generated in
// `crate::flags` (`JsxFlags::INTRINSIC_NAMED_ELEMENT`,
// `JsxReferenceKind::COMPONENT`, ...), so they are not redefined here.

// Go: checker/jsx.go:34 JsxElementLinks
#[derive(Clone, Debug, Default)]
pub struct JsxElementLinks {
    pub jsx_flags: JsxFlags,                          // Flags for the JSX element
    pub resolved_jsx_element_attributes_type: TypeId, // Resolved element attributes type of a JSX opening-like element
    pub jsx_namespace: SymbolId,                      // Resolved JSX namespace symbol for this node
    pub jsx_implicit_import_container: SymbolId, // Resolved module symbol the implicit JSX import of this file should refer to
    pub first_jsx_tag_in_file: Node,             // The first JSX tag in the file
}

// Go: checker/jsx.go:42 JsxNames
// PORT: Go `var JsxNames = struct{...}{...}`. Two spellings are provided so
// both `JsxNames.intrinsic_elements` (the value, like Go) and
// `JsxNames::INTRINSIC_ELEMENTS` (associated consts) work. The braced struct
// `JsxNames {}` lives in the type namespace and the const `JsxNames` lives in
// the value namespace, so they do not clash.
#[derive(Clone, Copy, Debug)]
pub struct JsxNamesValues {
    pub jsx: &'static str,
    pub intrinsic_elements: &'static str,
    pub element_class: &'static str,
    pub element_attributes_property_name_container: &'static str,
    pub element_children_attribute_name_container: &'static str,
    pub element: &'static str,
    pub element_type: &'static str,
    pub intrinsic_attributes: &'static str,
    pub intrinsic_class_attributes: &'static str,
    pub library_managed_attributes: &'static str,
}


pub struct JsxNames {}

impl JsxNames {
    pub const JSX: &'static str = "JSX";
    pub const INTRINSIC_ELEMENTS: &'static str = "IntrinsicElements";
    pub const ELEMENT_CLASS: &'static str = "ElementClass";
    pub const ELEMENT_ATTRIBUTES_PROPERTY_NAME_CONTAINER: &'static str =
        "ElementAttributesProperty";
    pub const ELEMENT_CHILDREN_ATTRIBUTE_NAME_CONTAINER: &'static str = "ElementChildrenAttribute";
    pub const ELEMENT: &'static str = "Element";
    pub const ELEMENT_TYPE: &'static str = "ElementType";
    pub const INTRINSIC_ATTRIBUTES: &'static str = "IntrinsicAttributes";
    pub const INTRINSIC_CLASS_ATTRIBUTES: &'static str = "IntrinsicClassAttributes";
    pub const LIBRARY_MANAGED_ATTRIBUTES: &'static str = "LibraryManagedAttributes";
}

// Go: checker/jsx.go:66 ReactNames
// PORT: same two spellings as `JsxNames`.
#[derive(Clone, Copy, Debug)]
pub struct ReactNamesValues {
    pub fragment: &'static str,
}


pub struct ReactNames {}

impl ReactNames {
    pub const FRAGMENT: &'static str = "Fragment";
}


// PORT: Go `func() (*diagnostics.Message, []any)` passed to the JSX child
// elaboration helpers. Stored in closures, so it is an `Rc<dyn Fn>`.

// Go: checker/jsx.go:369 JsxElaborationElement
// PORT: Go `createDiagnostic func(prop *ast.Node) *ast.Diagnostic` (nil-able)
// is `Option<Rc<dyn Fn(&mut Checker, Node) -> Diagnostic>>`.

// PORT: Go `generateJsxChildren` returns a lazy `iter.Seq[JsxElaborationElement]`.
// The Rust iterator keeps the same lazy order: each `next` call creates the
// number literal name types and elaborates only up to the next yielded child,
// so type creation interleaves with the consumer exactly as in Go.


