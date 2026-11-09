//! Go `ast/symbol.go`, `ast/diagnostic.go`, `ast/precedence.go`,
//! `ast/modifierflags.go`, `ast/flow.go`, `ast/checkflags.go`,
//! `ast/symbolflags.go`, `ast/functionflags.go`, `ast/positionmap.go` and
//! `ast/ids.go` (functions and non-flag types only; the flag consts live in
//! `crate::flags`).

use crate::prelude::*;

// ---------------------------------------------------------------------------
// symbol.go
// ---------------------------------------------------------------------------


// PORT: Go uses the byte 0xFE, which is invalid UTF-8. A Rust `String` holds
// it in the port form of Go strings (see `scanner_util::GO_STRING_MARKER`):
// U+FDD0 + U+10F7FE, the invalid byte unit for 0xFE. Source text with the
// byte 0xFE has the same port form, so it names the same internal symbols as
// in Go, and a real U+FFFE char stays an ordinary char. Go byte checks such as
// `name[0] == '\xFE'` port to `name.starts_with(INTERNAL_SYMBOL_NAME_PREFIX)`.
pub const INTERNAL_SYMBOL_NAME_PREFIX: &str = "\u{FDD0}\u{10F7FE}"; // Invalid as IdentifierName

pub const INTERNAL_SYMBOL_NAME_CALL: &str = "\u{FDD0}\u{10F7FE}call"; // Call signatures
pub const INTERNAL_SYMBOL_NAME_CONSTRUCTOR: &str = "\u{FDD0}\u{10F7FE}constructor"; // Constructor implementations
pub const INTERNAL_SYMBOL_NAME_NEW: &str = "\u{FDD0}\u{10F7FE}new"; // Constructor signatures
pub const INTERNAL_SYMBOL_NAME_INDEX: &str = "\u{FDD0}\u{10F7FE}index"; // Index signatures
pub const INTERNAL_SYMBOL_NAME_EXPORT_STAR: &str = "\u{FDD0}\u{10F7FE}export"; // Module export * declarations
pub const INTERNAL_SYMBOL_NAME_GLOBAL: &str = "\u{FDD0}\u{10F7FE}global"; // Global self-reference
pub const INTERNAL_SYMBOL_NAME_MISSING: &str = "\u{FDD0}\u{10F7FE}missing"; // Indicates missing symbol
pub const INTERNAL_SYMBOL_NAME_TYPE: &str = "\u{FDD0}\u{10F7FE}type"; // Anonymous type literal symbol
pub const INTERNAL_SYMBOL_NAME_OBJECT: &str = "\u{FDD0}\u{10F7FE}object"; // Anonymous object literal declaration
pub const INTERNAL_SYMBOL_NAME_JSX_ATTRIBUTES: &str = "\u{FDD0}\u{10F7FE}jsxAttributes"; // Anonymous JSX attributes object literal declaration
pub const INTERNAL_SYMBOL_NAME_CLASS: &str = "\u{FDD0}\u{10F7FE}class"; // Unnamed class expression
pub const INTERNAL_SYMBOL_NAME_FUNCTION: &str = "\u{FDD0}\u{10F7FE}function"; // Unnamed function expression
pub const INTERNAL_SYMBOL_NAME_COMPUTED: &str = "\u{FDD0}\u{10F7FE}computed"; // Computed property name declaration with dynamic name
pub const INTERNAL_SYMBOL_NAME_ASSIGNMENT_DECLARATION: &str = "\u{FDD0}\u{10F7FE}assignment"; // Assignment declarations
pub const INTERNAL_SYMBOL_NAME_INSTANTIATION_EXPRESSION: &str =
    "\u{FDD0}\u{10F7FE}instantiationExpression"; // Instantiation expressions
pub const INTERNAL_SYMBOL_NAME_IMPORT_ATTRIBUTES: &str = "\u{FDD0}\u{10F7FE}importAttributes";
pub const INTERNAL_SYMBOL_NAME_EXPORT_EQUALS: &str = "export="; // Export assignment symbol
pub const INTERNAL_SYMBOL_NAME_DEFAULT: &str = "default"; // Default export symbol (technically not wholly internal, but included here for usability)
pub const INTERNAL_SYMBOL_NAME_THIS: &str = "this";
pub const INTERNAL_SYMBOL_NAME_MODULE_EXPORTS: &str = "module.exports";

// Go: ast/symbol.go:72 SymbolName

// Go: ast/symbol.go:80 EscapeAllInternalSymbolNames
// EscapeAllInternalSymbolNames replaces internal symbol name markers ("\xFE") with "__".
// PORT: each byte 0xFE is the unit INTERNAL_SYMBOL_NAME_PREFIX in the port
// form. A plain text search could also match a U+FDD0 unit (U+FDD0 twice)
// followed by a real U+10F7FE char, so this reads the units.

// Go: ast/symbol.go:84 EscapeInternalSymbolName
// PORT: Go `strings.CutPrefix(name, "\xFE")` reads the first Go byte. The
// byte 0xFE is the unit `INTERNAL_SYMBOL_NAME_PREFIX` in the port form.
#[must_use]
pub fn escape_internal_symbol_name(name: &str) -> String {
    if let Some(rest) = name.strip_prefix(INTERNAL_SYMBOL_NAME_PREFIX) {
        return format!("__{rest}");
    }
    name.to_string()
}

// Go: ast/symbol.go:95 EscapeSymbolName
// EscapeSymbolName converts a binder symbol name into its escaped "__String"
// form. Internal names (prefixed with the "\xFE" sentinel) become "__"-prefixed,
// and user names that already begin with "__" gain an extra leading underscore
// so they can be distinguished from internal names.
#[must_use]
pub fn escape_symbol_name(name: &str) -> String {
    if let Some(rest) = name.strip_prefix(INTERNAL_SYMBOL_NAME_PREFIX) {
        return format!("__{rest}");
    }
    if name.starts_with("__") {
        return format!("_{name}");
    }
    name.to_string()
}

// ---------------------------------------------------------------------------
// diagnostic.go
// ---------------------------------------------------------------------------

/// Go `ast.RepopulateDiagnosticKind`: the kind of repopulation for a
/// diagnostic chain entry.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RepopulateDiagnosticKind(pub i32);

impl RepopulateDiagnosticKind {
    pub const MODE_MISMATCH: Self = Self(1); // RepopulateModeMismatch
    pub const MODULE_NOT_FOUND: Self = Self(2); // RepopulateModuleNotFound
}

/// Go `ast.RepopulateDiagnosticInfo`. It stores information needed to
/// recompute a diagnostic chain entry during incremental builds when the
/// program state may have changed.
/// PORT: Go `core.ResolutionMode` is an alias of `core.ModuleKind`.
#[derive(Clone, Debug, Default)]
pub struct RepopulateDiagnosticInfo {
    pub kind: RepopulateDiagnosticKind,
    pub module_reference: String,
    pub mode: ModuleKind,
    pub package_name: String,
}

// PORT: `core::Diagnostic` has no `messageKey` field. The message key is
// always `message.key()`.




// Go: diagnostics/diagnostics.go:129 Format
// PORT: also Go `diagnostics.Localize` with the default locale; the port has
// only the English messages. `Message::format` replaces the placeholders,
// and Go panics on a bad placeholder.

// Go: ast/diagnostic.go:166 NewDiagnosticFromSerialized
// PORT: Go keeps `message` nil and resolves the key lazily in `Localize`
// (panicking on an unknown key). `core::Diagnostic` needs the message, so a
// known key is resolved here. The key "" of a serialized external
// diagnostic (tsgo#4712: Go `SetExternalData` gives it its text next) is the
// Go nil message, and `localize` panics on it without a text, as Go does.
// An unknown key (a bad `.tsbuildinfo`) is a nil message that keeps the key
// (`unknown_key_message`), so `localize` panics when Go does, with Go's
// text, and the build info keeps the key.

// Go: ast/diagnostic.go:194 NewDiagnosticFromText (ts#63950)
// PORT: Go `file *SourceFile` is the SourceFile node (nil is `Node::NIL`).
// Go sets `message` to `diagnostics.NewAdHocMessage(text)` and leaves
// `messageKey` empty. The port reads the key from the message
// (`message_key`), so the ad hoc message has the key "" here, not the Go
// "-1": `MessageKey()` is "" as in Go, and no locale has either key. The
// message code is 0, not the Go -1 (`Message` codes are `u32`); the
// diagnostic keeps its own code, and nothing reads the message code.

// Go: diagnostics/diagnostics.go:164 NewAdHocMessage
// PORT: a diagnostic holds a `&'static Message`, so each distinct text is
// leaked once and shared (Go allocates a new message per call). The key is
// "" (see `new_diagnostic_from_text`, the only user).

// Go: ast/diagnostic.go:218 NewDiagnostic

// Go: ast/diagnostic.go:232 NewDiagnosticChain

// Go: ast/diagnostic.go:239 NewCompilerDiagnostic
// PORT: Go `core.UndefinedTextRange()` is `TextRange{-1, -1}`.

// Go: ast/diagnostic.go:247 NewExternalDiagnostic (tsgo#4712)
// NewExternalDiagnostic creates a diagnostic reported by an external source such as a content mapper.
// The message text is already localized (the external source owns localization) and the code is shown
// with the given source prefix (e.g. "vue") instead of "TS". The location refers to the file's original,
// untransformed content.
// PORT: Go `file *SourceFile` is the SourceFile node (nil is `Node::NIL`).
// The Go nil message is `NIL_MESSAGE`.



// Go `slices.SortStableFunc(list, CompareDiagnostics)` on a list of
// positions in `diagnostics`, with the paths of `stored_file_names`
// (`stored_diagnostic_path`).



// Go: ast/diagnostic.go:312 diagnosticLocationKey (#4825)
// PORT: `path` is the file name (see `DiagnosticsCollection`). Go `loc` is
// `pos` and `end`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct DiagnosticLocationKey {
    path: &'static str,
    pos: i32,
    end: i32,
    code: i32,
}

// Go: ast/diagnostic.go:318 getDiagnosticLocationKey (#4825)
// PORT: `get_diagnostic_path` is the Go path part: the file name, or "" when
// there is no file.

// Go: ast/diagnostic.go:390 getDiagnosticPath
// PORT: returns `&'static str` (file names live for the program).

// Go: ast/diagnostic.go:397 EqualDiagnostics


// Go: ast/diagnostic.go:405 EqualDiagnosticsNoRelatedInfo



// Go: ast/diagnostic.go:419 getDiagnosticMessageIdentity (#4825, tsgo#4712)
// PORT: a port diagnostic has its message, or `NIL_MESSAGE` for the Go nil
// message, whose text and key are both "" (see
// `new_diagnostic_from_serialized`). Go `message.String()` is the message
// text.

// Go: ast/diagnostic.go:429 equalMessageChain

// Go `slices.Compare` / `strings.Compare` as -1, 0, 1.
fn ordering_to_int(ordering: std::cmp::Ordering) -> i32 {
    match ordering {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

// Go: ast/diagnostic.go:438 compareMessageChainSize

// Go: ast/diagnostic.go:452 compareMessageChainContent

// Go: ast/diagnostic.go:468 compareRelatedInfo
// PORT: Go `getDiagnosticPath` is read by `path` (`compare_diagnostics_by`).

// Go: ast/diagnostic.go:482 CompareDiagnostics




// ---------------------------------------------------------------------------
// precedence.go (OperatorPrecedence, OperatorPrecedenceFlags and
// TypePrecedence consts are in `crate::flags`)
// ---------------------------------------------------------------------------

// Go: ast/precedence.go:189 getOperator

// Go: ast/precedence.go:203 GetExpressionPrecedence
// Gets the precedence of an expression

// Go: ast/precedence.go:223 GetOperatorPrecedence
// Gets the precedence of an operator
#[must_use]
pub fn get_operator_precedence(
    node_kind: SyntaxKind,
    operator_kind: SyntaxKind,
    flags: OperatorPrecedenceFlags,
) -> OperatorPrecedence {
    match node_kind {
        SyntaxKind::SpreadElement => OperatorPrecedence::SPREAD,
        SyntaxKind::YieldExpression => OperatorPrecedence::YIELD,
        // !!! By necessity, this differs from the old compiler to better align with ParenthesizerRules. consider backporting
        SyntaxKind::ArrowFunction => OperatorPrecedence::ASSIGNMENT,
        SyntaxKind::ConditionalExpression => OperatorPrecedence::CONDITIONAL,
        SyntaxKind::BinaryExpression => match operator_kind {
            SyntaxKind::CommaToken => OperatorPrecedence::COMMA,

            SyntaxKind::EqualsToken
            | SyntaxKind::PlusEqualsToken
            | SyntaxKind::MinusEqualsToken
            | SyntaxKind::AsteriskAsteriskEqualsToken
            | SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PercentEqualsToken
            | SyntaxKind::LessThanLessThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
            | SyntaxKind::AmpersandEqualsToken
            | SyntaxKind::CaretEqualsToken
            | SyntaxKind::BarEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken => OperatorPrecedence::ASSIGNMENT,

            _ => get_binary_operator_precedence(operator_kind),
        },
        // TODO: Should prefix `++` and `--` be moved to the `Update` precedence?
        SyntaxKind::TypeAssertionExpression
        | SyntaxKind::NonNullExpression
        | SyntaxKind::PrefixUnaryExpression
        | SyntaxKind::TypeOfExpression
        | SyntaxKind::VoidExpression
        | SyntaxKind::DeleteExpression
        | SyntaxKind::AwaitExpression => OperatorPrecedence::UNARY,

        SyntaxKind::PostfixUnaryExpression => OperatorPrecedence::UPDATE,

        // !!! By necessity, this differs from the old compiler to better align with ParenthesizerRules. consider backporting
        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
            if flags.intersects(OperatorPrecedenceFlags::OPTIONAL_CHAIN) {
                return OperatorPrecedence::OPTIONAL_CHAIN;
            }
            OperatorPrecedence::MEMBER
        }

        SyntaxKind::CallExpression => {
            if flags.intersects(OperatorPrecedenceFlags::OPTIONAL_CHAIN) {
                return OperatorPrecedence::OPTIONAL_CHAIN;
            }
            OperatorPrecedence::MEMBER
        }

        // !!! By necessity, this differs from the old compiler to better align with ParenthesizerRules. consider backporting
        SyntaxKind::NewExpression => {
            if flags.intersects(OperatorPrecedenceFlags::NEW_WITHOUT_ARGUMENTS) {
                return OperatorPrecedence::LEFT_HAND_SIDE;
            }
            OperatorPrecedence::MEMBER
        }

        // !!! By necessity, this differs from the old compiler to better align with ParenthesizerRules. consider backporting
        SyntaxKind::TaggedTemplateExpression
        | SyntaxKind::MetaProperty
        | SyntaxKind::ExpressionWithTypeArguments => OperatorPrecedence::MEMBER,

        SyntaxKind::AsExpression | SyntaxKind::SatisfiesExpression => {
            OperatorPrecedence::RELATIONAL
        }

        SyntaxKind::ThisKeyword
        | SyntaxKind::SuperKeyword
        | SyntaxKind::ImportKeyword
        | SyntaxKind::Identifier
        | SyntaxKind::PrivateIdentifier
        | SyntaxKind::NullKeyword
        | SyntaxKind::TrueKeyword
        | SyntaxKind::FalseKeyword
        | SyntaxKind::NumericLiteral
        | SyntaxKind::BigIntLiteral
        | SyntaxKind::StringLiteral
        | SyntaxKind::ArrayLiteralExpression
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ClassExpression
        | SyntaxKind::RegularExpressionLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::TemplateExpression
        | SyntaxKind::OmittedExpression
        | SyntaxKind::JsxElement
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::JsxFragment
        | SyntaxKind::MissingDeclaration => OperatorPrecedence::PRIMARY,

        // !!! By necessity, this differs from the old compiler to support emit. consider backporting
        SyntaxKind::ParenthesizedExpression => OperatorPrecedence::PARENTHESES,

        _ => OperatorPrecedence::INVALID,
    }
}

// Go: ast/precedence.go:336 GetBinaryOperatorPrecedence
// Gets the precedence of a binary operator
#[must_use]
pub fn get_binary_operator_precedence(operator_kind: SyntaxKind) -> OperatorPrecedence {
    match operator_kind {
        SyntaxKind::QuestionQuestionToken => return OperatorPrecedence::COALESCE,
        SyntaxKind::BarBarToken => return OperatorPrecedence::LOGICAL_OR,
        SyntaxKind::AmpersandAmpersandToken => return OperatorPrecedence::LOGICAL_AND,
        SyntaxKind::BarToken => return OperatorPrecedence::BITWISE_OR,
        SyntaxKind::CaretToken => return OperatorPrecedence::BITWISE_XOR,
        SyntaxKind::AmpersandToken => return OperatorPrecedence::BITWISE_AND,
        SyntaxKind::EqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken => return OperatorPrecedence::EQUALITY,
        SyntaxKind::LessThanToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::InKeyword
        | SyntaxKind::AsKeyword
        | SyntaxKind::SatisfiesKeyword => return OperatorPrecedence::RELATIONAL,
        SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken => return OperatorPrecedence::SHIFT,
        SyntaxKind::PlusToken | SyntaxKind::MinusToken => return OperatorPrecedence::ADDITIVE,
        SyntaxKind::AsteriskToken | SyntaxKind::SlashToken | SyntaxKind::PercentToken => {
            return OperatorPrecedence::MULTIPLICATIVE;
        }
        SyntaxKind::AsteriskAsteriskToken => return OperatorPrecedence::EXPONENTIATION,
        _ => {}
    }
    // -1 is lower than all other precedences.  Returning it will cause binary expression
    // parsing to stop.
    OperatorPrecedence::INVALID
}

// Go: ast/precedence.go:370 GetLeftmostExpression
// Gets the leftmost expression of an expression, e.g. `a` in `a.b`, `a[b]`, `a++`, `a+b`, `a?b:c`, `a as B`, etc.

// Go: ast/precedence.go:655 GetTypeNodePrecedence
// Gets the precedence of a TypeNode

// ---------------------------------------------------------------------------
// modifierflags.go, checkflags.go, symbolflags.go: consts only (in
// `crate::flags`).
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// flow.go (FlowFlags consts are in `crate::flags`; `FlowNode` is in
// `crate::core`, with `antecedents: Vec<FlowNodeId>` in place of Go `FlowList`)
// ---------------------------------------------------------------------------

/// Go `ast.FlowSwitchClauseData` (synthetic AST node for
/// `FlowFlags::SWITCH_CLAUSE`).
/// PORT: Go wraps this in a synthetic `*Node` of `KindUnknown` stored in
/// `FlowNode.Node`. `core::FlowNode.node` is a handle into parsed files and
/// cannot hold a synthetic node, so this is a plain value. The owner of the
/// flow node storage decides where it lives.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FlowSwitchClauseData {
    pub switch_statement: Node,
    pub clause_start: i32, // Start index of case/default clause range
    pub clause_end: i32,   // End index of case/default clause range
}

// Go: ast/flow.go:50 NewFlowSwitchClauseData
// PORT: returns the data value instead of a synthetic `*Node` (see the struct).
#[must_use]
pub fn new_flow_switch_clause_data(
    switch_statement: Node,
    clause_start: i32,
    clause_end: i32,
) -> FlowSwitchClauseData {
    FlowSwitchClauseData {
        switch_statement,
        clause_start,
        clause_end,
    }
}

impl FlowSwitchClauseData {
    // Go: ast/flow.go:58 IsEmpty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.clause_start == self.clause_end
    }
}

/// Go `ast.FlowReduceLabelData` (synthetic AST node for
/// `FlowFlags::REDUCE_LABEL`).
/// PORT: plain value, like `FlowSwitchClauseData`. Go `*FlowList` becomes
/// `Vec<FlowNodeId>` in the same order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FlowReduceLabelData {
    pub target: FlowNodeId,           // Target label
    pub antecedents: Vec<FlowNodeId>, // Temporary antecedent list
}

// Go: ast/flow.go:70 NewFlowReduceLabelData
// PORT: returns the data value instead of a synthetic `*Node` (see the struct).
#[must_use]
pub fn new_flow_reduce_label_data(
    target: FlowNodeId,
    antecedents: Vec<FlowNodeId>,
) -> FlowReduceLabelData {
    FlowReduceLabelData {
        target,
        antecedents,
    }
}

// ---------------------------------------------------------------------------
// functionflags.go (FunctionFlags consts are in `crate::flags`)
// ---------------------------------------------------------------------------

// Go: ast/functionflags.go:13 GetFunctionFlags

// ---------------------------------------------------------------------------
// positionmap.go
// ---------------------------------------------------------------------------

/// Go `ast.PositionMap`: bidirectional mapping between UTF-8 byte offsets
/// (used by Go) and UTF-16 code unit offsets (used by JavaScript/TypeScript).
///
/// For ASCII-only text, the two are identical. For text containing non-ASCII
/// characters, the offsets diverge because multi-byte UTF-8 sequences map to
/// different numbers of UTF-16 code units:
///   - U+0000..U+007F:   1 byte  in UTF-8, 1 code unit  in UTF-16
///   - U+0080..U+07FF:   2 bytes in UTF-8, 1 code unit  in UTF-16
///   - U+0800..U+FFFF:   3 bytes in UTF-8, 1 code unit  in UTF-16
///   - U+10000..U+10FFFF: 4 bytes in UTF-8, 2 code units in UTF-16 (surrogate pair)
#[derive(Clone, Debug, Default)]
pub struct PositionMap {
    /// True if the text contains only ASCII characters, meaning UTF-8 byte
    /// offsets and UTF-16 code unit offsets are identical.
    pub ascii_only: bool,
    /// For each multi-byte character: the UTF-8 byte offset after it and the
    /// cumulative delta (utf8Offset - utf16Offset) through it. This allows
    /// O(log n) conversion in either direction.
    pub entries: Vec<PositionMapEntry>,
    /// PORT: the port offset of each rune whose Go bytes are in more than
    /// one unit of the port form (`SPLIT_RUNE_LEN`). Usually empty.
    split_runes: Vec<i32>,
}

/// PORT: the port size of the only Go rune that is more than one unit of the
/// port form: a WTF-8 lone surrogate in source text, which is 3 invalid byte
/// units of 7 bytes (see `scanner_util::GO_STRING_MARKER`). Every other
/// rune is at most one unit of at most 7 bytes. Go reads its 3 bytes as one
/// rune (`DecodeJSStringRune`), and a position k bytes into it is the end of
/// its k-th unit here.
const SPLIT_RUNE_UNIT_LEN: i32 = 7;
const SPLIT_RUNE_LEN: i32 = 3 * SPLIT_RUNE_UNIT_LEN;

/// Go `ast.positionMapEntry`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PositionMapEntry {
    pub utf8_pos: i32, // UTF-8 byte offset AFTER this multi-byte character
    pub delta: i32,    // cumulative (utf8 - utf16) offset difference after this character
}

// Go: ast/positionmap.go:40 ComputePositionMap
// ComputePositionMap builds a PositionMap for the given text.
// PORT: `text` is the port form of the Go text (see
// `scanner_util::GO_STRING_MARKER`), and the "UTF-8" offsets are offsets in
// it. `decode_go_js_string_rune` reads one Go `DecodeJSStringRune` rune and
// gives its size in the port form, so each entry holds a port offset and the
// UTF-16 length of the Go rune. An invalid byte and a WTF-8 lone surrogate
// are each one UTF-16 unit, as in Go. A WTF-8 lone surrogate is 3 units
// here; `split_runes` keeps its offset for the positions inside it.

impl PositionMap {
    // Go: ast/positionmap.go:64 IsAsciiOnly
    // IsAsciiOnly returns true if the text is ASCII-only,
    // meaning UTF-8 and UTF-16 offsets are identical.
    #[must_use]
    pub fn is_ascii_only(&self) -> bool {
        self.ascii_only
    }

    // Go: ast/positionmap.go:69 UTF8ToUTF16
    // UTF8ToUTF16 converts a UTF-8 byte offset to a UTF-16 code unit offset.


    // Go: ast/positionmap.go:91 UTF16ToUTF8
    // UTF16ToUTF8 converts a UTF-16 code unit offset to a UTF-8 byte offset.
    #[must_use]
    pub fn utf16_to_utf8(&self, utf16_offset: i32) -> i32 {
        if self.ascii_only {
            return utf16_offset;
        }
        // We need the last entry where (utf8Pos - delta) <= utf16Offset.
        // (utf8Pos - delta) is the UTF-16 offset of that entry's character.
        let (mut lo, mut hi) = (0usize, self.entries.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let utf16_pos = self.entries[mid].utf8_pos - self.entries[mid].delta;
            if utf16_pos <= utf16_offset {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo == 0 {
            return utf16_offset;
        }
        utf16_offset + self.entries[lo - 1].delta
    }
}

// ---------------------------------------------------------------------------
// ids.go
// ---------------------------------------------------------------------------

// PORT: Go `ast.NodeId` and `ast.SymbolId` are plain `uint64` ids. The port
// uses the `core::Node` and `core::SymbolId` handles instead, so there is
// nothing to define here.

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::AssertUnwindSafe;

    // Go prints `%v` of the Kind in this panic (ast/precedence.go:715
    // `panic(fmt.Sprintf("unhandled TypeNode: %v", n.Kind))`), which is
    // `Kind.String()`. The API's printNode reaches it (gaps2a skeptic repro
    // r-typenode).
    #[test]
    fn unhandled_type_node_panic_names_the_go_kind() {
        let f = NodeFactory::new();
        let name = f.new_identifier("x");
        let payload = std::panic::catch_unwind(AssertUnwindSafe(|| get_type_node_precedence(name)))
            .expect_err("no panic");
        assert_eq!(
            payload.downcast_ref::<String>().map(String::as_str),
            Some("unhandled TypeNode: KindIdentifier")
        );
    }
}
