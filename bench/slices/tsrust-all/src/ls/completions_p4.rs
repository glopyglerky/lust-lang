use crate::ls::prelude::*;

// Port of Go `ls/completions.go` lines 4879-6286: completion item resolve
// and details, import statement completion info, JSDoc tag and parameter
// completions, exhaustive switch case snippets and the snippet printer.

use crate::frontend::core_textchange::{TextChange, };


// Go: completions.go:5512 getCompletionDocumentationFormat


// Go: completions.go:5609 detailsData
// PORT: Go `request *completionData` is `Option<CompletionData>`, and
// `cases *struct{}` is `Option<()>`.

// Go: completions.go:5616 symbolDetails
// PORT: Go shares the `*symbolOriginInfo` pointer; Rust clones the origin.
#[derive(Clone, Debug, Default)]
pub struct SymbolDetails {
    pub symbol: SymbolId,
    pub location: Node,
    pub origin: Option<SymbolOriginInfo>,
    pub previous_token: Node,
    pub context_token: Node,
    pub jsx_initializer: JsxInitializer,
    pub is_type_only_location: bool,
}


// Go: completions.go:5698 createSimpleDetails

// Go: completions.go:5706 createCompletionDetails

// Go: completions.go:5727 codeAction
// PORT: Go's unexported `codeAction` is unused. It stays private because the
// exported `CodeAction` (codeactions.go) has the same Rust name in package `ls`.
#[allow(dead_code)]
struct CodeAction {
    // Description of the code action to display in the UI of the editor
    description: String,
    // Text changes to apply to each file as part of the code action
    changes: Vec<lsproto::TextEdit>,
}


// Go: completions.go:5879 couldBeTypeOnlyImportSpecifier

// Go: completions.go:5883 canCompleteFromNamedBindings

// Go: completions.go:5911 getPotentiallyInvalidImportSpecifier
// Tries to identify the first named import that is not really a named import, but rather
// just parser recovery for a situation like:
//
//	import { Foo|
//	interface Bar {}
//
// in which `Foo`, `interface`, and `Bar` are all parsed as import specifiers. The caller
// will also check if this token is on a separate line from the rest of the import.

// Go: completions.go:5921 isModuleSpecifierMissingOrEmpty

// Go: completions.go:5935 hasDocComment

// Go: completions.go:5941 getJSDocTagAtPosition
// Get the corresponding JSDocTag node if the position is in a JSDoc comment

// Go: completions.go:5953 tryGetTypeExpressionFromTag

// Go: completions.go:5971 isTagWithTypeExpression


// Go: completions.go:6005 jsDocTagNames
pub static JS_DOC_TAG_NAMES: &[&str] = &[
    "abstract",
    "access",
    "alias",
    "argument",
    "async",
    "augments",
    "author",
    "borrows",
    "callback",
    "class",
    "classdesc",
    "constant",
    "constructor",
    "constructs",
    "copyright",
    "default",
    "deprecated",
    "description",
    "emits",
    "enum",
    "event",
    "example",
    "exports",
    "extends",
    "external",
    "field",
    "file",
    "fileoverview",
    "fires",
    "function",
    "generator",
    "global",
    "hideconstructor",
    "host",
    "ignore",
    "implements",
    "import",
    "inheritdoc",
    "inner",
    "instance",
    "interface",
    "kind",
    "lends",
    "license",
    "link",
    "linkcode",
    "linkplain",
    "listens",
    "member",
    "memberof",
    "method",
    "mixes",
    "module",
    "name",
    "namespace",
    "overload",
    "override",
    "package",
    "param",
    "private",
    "prop",
    "property",
    "protected",
    "public",
    "readonly",
    "requires",
    "returns",
    "satisfies",
    "see",
    "since",
    "static",
    "summary",
    "template",
    "this",
    "throws",
    "todo",
    "tutorial",
    "type",
    "typedef",
    "var",
    "variation",
    "version",
    "virtual",
    "yields",
];

// PORT: Go `sync.OnceValue` package vars become lazily built thread-local
// values (all language-service work runs on the dispatch thread).

// Go: completions.go:6118 getJSDocTagNameCompletions

// Go: completions.go:6122 getJSDocTagCompletions

// Go: completions.go:6126 getJSDocParameterCompletions

// Go: completions.go:6269 getJSDocParamAnnotation
// PORT: Go `tabstopCounter *int` is never nil at a call site, so it is
// `&mut i32`; the Go `debug.Assert(tabstopCounter != nil)` always holds.

// Go: completions.go:6352 getJSDocParamNameWithInitializer
// PORT: Go `strings.TrimSpace` and Rust `str::trim` both trim Unicode
// White_Space.

// Go: completions.go:6360 generateJSDocParamTagsForDestructuring

// Go: completions.go:6400 jsDocParamPatternWorker

// Go: completions.go:6469 jsDocParamElementWorker
// Assumes binding element is inside object binding pattern.
// We can't deeply annotate an array binding pattern.

// Go: completions.go:6527 getJSDocParameterNameCompletions


// Go: completions.go:6706 typeNodeToExpression

// Go: completions.go:6768 entityNameToExpression

// Go: completions.go:6785 snippetPrinter
// PORT: Go keeps `baseWriter` and the writer that embeds it as two pointers
// to one `ChangeTrackerWriter`. In Rust the snippet writer owns it, so Go
// `p.baseWriter` is `p.writer.borrow().change_tracker_writer`. The printer
// writes through `Rc<RefCell<dyn EmitTextWriter>>`, so the writer is shared.
// Go `factory` is `emitContext.Factory.AsNodeFactory()`, a pointer into the
// context; here it is the method `factory()`.


// Go: completions.go:6836 createSnippetPrinter
// PORT: a nil Go `emitContext` is `None`.

// Go: completions.go:6855 snippetEmitTextWriter
// Override base writer methods to perform snippet escaping.
// PORT: Go embeds `*printer.ChangeTrackerWriter`; here it is the owned field
// `change_tracker_writer`. The `EmitTextWriter` impl below forwards every
// method that Go does not override.
pub struct SnippetEmitTextWriter {
    pub change_tracker_writer: ChangeTrackerWriter,
    pub escapes: Vec<TextChange>,
}

impl SnippetEmitTextWriter {
    // Go: completions.go:6888 escapingWrite
    // The formatter/scanner will have issues with snippet-escaped text,
    // so instead of writing the escaped text directly to the writer,
    // generate a set of changes that can be applied to the unescaped text
    // to escape it post-formatting.
}

