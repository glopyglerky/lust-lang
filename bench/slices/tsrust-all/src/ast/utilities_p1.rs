//! Port of typescript-go `internal/ast/utilities.go` lines 1-905.

use crate::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};

// Atomic ids

// PORT: Go keeps `nextNodeId`/`nextSymbolId` as process-wide atomics and
// stores the lazily assigned id on the node or symbol. Our AST and symbol
// arena entries have no id slot, so the lazily assigned ids live in
// thread-local maps keyed by handle. Ids are still assigned on first request
// in request order, like Go.

/// A symbol id as an id table keeps it: 0 for no id yet (Go ids start at
/// 1), `BIG_ID` for an id of `BIG_ID` or more (in the table's `big` map),
/// else the id.
// PERF: 4 bytes per symbol. Each checker gives an id to most symbols that it
// reads (Go `GetSymbolId` in `symbolArenaLinkStore`), up to 2 million of its
// own on zod. Ids count per thread from 1, so a CLI run stays far below
// `BIG_ID`.
type IdCell = u32;


/// The ids of the binder lineage symbols on one thread, by lineage index, in
/// chunks of the symbol chunk size of `SymbolArena` (`LINEAGE_ID_CHUNK`),
/// as `IdCell`s. A chunk is made at the first id in it.
///
/// lsshells M3d: when `program::Lineage` frees the chunks of a dead file
/// version (`free_lineage_symbol_ids`), each thread frees its id chunks of
/// that range the next time it makes a chunk. Only a new chunk grows the
/// table, so the table does not grow with edits. A later id read of a freed
/// chunk panics, as a read of the symbol in the lineage does.
#[derive(Clone, Debug, Default)]
struct LineageIds {
    /// None: no id in this chunk yet, or a freed chunk (`freed`).
    chunks: Vec<Option<Box<[IdCell; LINEAGE_ID_CHUNK]>>>,
    /// The ids from `BIG_ID` on, by lineage index.
    big: FxHashMap<usize, u64>,
    /// The freed chunks, one bit each.
    freed: Vec<u64>,
    /// The entries of `FREED_LINEAGE` that this table has freed.
    seen: usize,
}

/// The symbols of one `SymbolArena` chunk (core.rs `COW_CHUNK_LEN`), so a
/// freed lineage range frees whole id chunks. A test checks that the two
/// agree (`lineage_id_chunks_are_symbol_chunks`).
// PORT: not shared with core.rs, which is a lib blob key.
const LINEAGE_ID_SHIFT: usize = 8;
const LINEAGE_ID_CHUNK: usize = 1 << LINEAGE_ID_SHIFT;

/// A lineage range that `program::Lineage` freed: the symbol chunks from
/// `first` to `end` of file version `file`.
#[derive(Clone, Copy, Debug)]
struct FreedLineage {
    file: usize,
    first: u32,
    end: u32,
}

/// The lineage ranges freed so far, in order. Each `LineageIds` frees the
/// entries after its `seen` count.
// PERF: one entry per dead file version, so it grows by 16 bytes per edit.
static FREED_LINEAGE: Mutex<Vec<FreedLineage>> = Mutex::new(Vec::new());

/// The length of `FREED_LINEAGE`, so a table with nothing to free does not
/// lock it.
static FREED_LINEAGE_COUNT: AtomicUsize = AtomicUsize::new(0);


impl LineageIds {



}


/// The ids of the symbols that checker arenas add on one thread, by arena
/// key. Such a symbol has an index that other checkers and later binds use
/// for other symbols, so its id cannot live in `SYMBOL_IDS`. A checker
/// worker has one arena; a thread that runs checkers of several programs
/// (the language server's) switches between them.
struct OwnSymbolIds {
    /// The arena whose ids are in `ids`, or 0.
    key: u32,
    /// By place among the arena's own symbols (`SymbolIds::own_place`), as
    /// `IdCell`s. The places are dense until a catch-up of the arena passes
    /// its own tail. After that, each odd chunk that the lineage passed is a
    /// gap of 256 places (1 KiB) here.
    ids: Vec<IdCell>,
    /// The ids of the other arenas.
    others: FxHashMap<u32, Vec<IdCell>>,
    /// The ids from `BIG_ID` on, by arena and place.
    big: FxHashMap<(u32, usize), u64>,
    /// The arenas whose ids stay after the arena drops
    /// (`keep_own_symbol_ids`).
    kept: Vec<u32>,
}

impl OwnSymbolIds {



    /// Makes arena `key` the one in `ids`.
    #[cold]
    #[inline(never)]
    fn switch_to(&mut self, key: u32) {
        let ids = self.others.remove(&key).unwrap_or_default();
        let previous = std::mem::replace(&mut self.ids, ids);
        if self.key != 0 {
            self.others.insert(self.key, previous);
        }
        self.key = key;
    }
}








/// The symbol ids that the checker of one program left on its thread: the
/// next id and the ids of the binder lineage symbols. The ids of the
/// checker's own symbols die with the checker.
// PORT: Go has one symbol id counter per process, and a bound file keeps
// the ids of its symbols in every later program. A one-checker
// `--singleThreaded` pool hands these ids to its loading thread when the
// program is released (`program::CheckerPool::stop`), so the next program
// of the process (`tsc -b`, watch) counts on from them.
pub struct SymbolIdCarry {
    next_symbol_id: u64,
    symbol_ids: LineageIds,
}




/// wasm: the ids of one checker of the inline checker pool
/// (`program::WorkerState`), which a native checker worker keeps in its
/// own thread. A job swaps them in (`swap_id_state`).
// PORT: not in Go.
#[cfg(target_family = "wasm")]
pub struct IdState {
    next_node_id: u64,
    next_symbol_id: u64,
    node_ids: PerFileMap<u64>,
    symbol_ids: LineageIds,
    own_symbol_ids: OwnSymbolIds,
}

#[cfg(target_family = "wasm")]
impl From<IdSeed> for IdState {
    /// The ids of a new worker, as `install_id_seed` sets them.
    fn from(seed: IdSeed) -> Self {
        IdState {
            next_node_id: seed.next_node_id,
            next_symbol_id: seed.next_symbol_id,
            node_ids: seed.node_ids,
            symbol_ids: seed.symbol_ids,
            own_symbol_ids: OwnSymbolIds::new(),
        }
    }
}

/// wasm: swaps the ids of this thread with `state`.
// PORT: not in Go.
#[cfg(target_family = "wasm")]
pub fn swap_id_state(state: &mut IdState) {
    NEXT_NODE_ID.with(|next| state.next_node_id = next.replace(state.next_node_id));
    NEXT_SYMBOL_ID.with(|next| state.next_symbol_id = next.replace(state.next_symbol_id));
    NODE_IDS.with(|ids| std::mem::swap(&mut *ids.borrow_mut(), &mut state.node_ids));
    SYMBOL_IDS.with(|ids| std::mem::swap(&mut *ids.borrow_mut(), &mut state.symbol_ids));
    OWN_SYMBOL_IDS.with(|own| std::mem::swap(&mut *own.borrow_mut(), &mut state.own_symbol_ids));
}


// Go: ast/utilities.go:22 GetNodeId
// PORT: Go `ast.NodeId` is a `uint64`; returned here as `u64`.

// Go: ast/utilities.go:34 GetSymbolId
// PORT: Go `ast.SymbolId` is a `uint64`; returned here as `u64`. A symbol
// handle is an index into `symbols`, which gives the slot where the id is
// kept (see `SymbolArena::id_slot`).

// Go: ast/utilities.go:46 GetSymbolTable
// PORT: Go takes `*SymbolTable` and allocates the map in place. The arena
// must be mutable to allocate a table.

// Go: ast/utilities.go:53 GetMembers

// Go: ast/utilities.go:57 GetExports

// Go: ast/utilities.go:61 GetLocals
// PORT: Go writes `container.LocalsContainerData().Locals`. Installed binder
// data (`Node::locals`) is not written again, so the binder passes the
// mutable per-node bind data of the file it is binding, indexed by
// `NodeId::index()`.

// Determines if a node is missing (either `nil` or empty)
// Go: ast/utilities.go:66 NodeIsMissing

// Determines if a node is present
// Go: ast/utilities.go:71 NodeIsPresent

// Determines if a node contains synthetic positions
// Go: ast/utilities.go:76 NodeIsSynthesized

// Go: ast/utilities.go:80 RangeIsSynthesized
pub fn range_is_synthesized(loc: TextRange) -> bool {
    position_is_synthesized(loc.pos()) || position_is_synthesized(loc.end())
}

// Determines whether a position is synthetic
// Go: ast/utilities.go:85 PositionIsSynthesized
pub fn position_is_synthesized(pos: i32) -> bool {
    pos < 0
}

// Go: ast/utilities.go:89 FindLastVisibleNode

// Go: ast/utilities.go:100 NodeKindIs

// Go: ast/utilities.go:104 IsModifier

// Go: ast/utilities.go:108 IsModifierLike

// Go: ast/utilities.go:112 IsCompoundAssignment
pub fn is_compound_assignment(token: SyntaxKind) -> bool {
    (token as u16) >= (SyntaxKind::FIRST_COMPOUND_ASSIGNMENT as u16)
        && (token as u16) <= (SyntaxKind::LAST_COMPOUND_ASSIGNMENT as u16)
}

/// The fields of a BinaryExpression, read once: Go `node.AsBinaryExpression()`
/// and its fields, plus `OperatorToken.Kind`. The binder reads them through
/// this view (`Binder::binary_view`) in `bind` and passes it on.
// PERF: binderview1 (cliperf1 rank 4a). Each accessor read (`left`,
// `operator_token`, `right`, `type_`) looks the node up again: about 45
// instructions and 2 file block lookups, and the binder made about 7
// `operator_token` reads per BinaryExpression. A view loads the node data
// once and resolves the child ids with no lookup (`FrozenIds`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BinaryView {
    pub left: Node,
    pub operator_token: Node,
    /// Go `OperatorToken.Kind`.
    pub operator: SyntaxKind,
    pub right: Node,
    pub type_: Node,
}

impl BinaryView {



}

// Go: ast/utilities.go:116 IsAssignmentExpression

// Go: ast/utilities.go:125 GetRightMostAssignedExpression

// Go: ast/utilities.go:132 IsDestructuringAssignment

// Go: ast/utilities.go:140 IsObjectBindingOrAssignmentElement

// Go: ast/utilities.go:151 IsArrayBindingOrAssignmentElement

// Go: ast/utilities.go:166 IsBindingPattern

// Go: ast/utilities.go:170 IsForInOrOfStatement

// A node is an assignment target if it is on the left hand side of an '=' token, if it is parented by a property
// assignment in an object literal that is an assignment target, or if it is parented by an array literal that is
// an assignment target. Examples include 'a = xxx', '{ p: a } = xxx', '[{ a }] = xxx'.
// (Note that `p` is not a target in the above examples, only `a`.)
// Go: ast/utilities.go:178 IsAssignmentTarget

// Returns the BinaryExpression, PrefixUnaryExpression, PostfixUnaryExpression, or ForInOrOfStatement that references
// the given node as an assignment target
// Go: ast/utilities.go:184 GetAssignmentTarget
// PERF: U4 (CH7). A step inside a published store reads the parent and its
// kind with one store lookup (`frozen_store_parent_kind`).

// Go: ast/utilities.go:228 IsLogicalBinaryOperator
pub fn is_logical_binary_operator(token: SyntaxKind) -> bool {
    token == SyntaxKind::BarBarToken || token == SyntaxKind::AmpersandAmpersandToken
}

// Go: ast/utilities.go:232 IsLogicalOrCoalescingBinaryOperator
pub fn is_logical_or_coalescing_binary_operator(token: SyntaxKind) -> bool {
    is_logical_binary_operator(token) || token == SyntaxKind::QuestionQuestionToken
}

// Go: ast/utilities.go:236 IsLogicalOrCoalescingBinaryExpression

// Go: ast/utilities.go:240 IsLogicalOrCoalescingAssignmentExpression

// Go: ast/utilities.go:244 IsLogicalExpression

// Go: ast/utilities.go:256 IsAccessor

// Go: ast/utilities.go:260 IsPropertyNameLiteral

// Go: ast/utilities.go:271 IsMemberName

// Go: ast/utilities.go:275 IsEntityName

// Go: ast/utilities.go:279 IsPropertyName

// Return true if the given identifier is classified as an IdentifierName by inspecting the parent of the node
// Go: ast/utilities.go:292 IsIdentifierName

// Go: ast/utilities.go:310 IsPushOrUnshiftIdentifier

// Go: ast/utilities.go:315 IsBooleanLiteral

// Go: ast/utilities.go:319 IsLiteralExpression

// Go: ast/utilities.go:323 IsStringLiteralLike

// Go: ast/utilities.go:331 IsStringOrNumericLiteralLike
// PERF: binderview1. The kind is read once (see `is_access_expression`).

// Go: ast/utilities.go:335 IsSignedNumericLiteral

// Determines if a node is part of an OptionalChain
// Go: ast/utilities.go:344 IsOptionalChain
// PERF: U4 (CH7). The kind test comes first (both tests are pure, so the
// result is the same) and `OPTIONAL_CHAIN` is a parser bit
// (`Node::parser_flags`), so most nodes load neither their flags nor their
// binder data.

// Go: ast/utilities.go:357 getQuestionDotToken

// Determines if node is the root expression of an OptionalChain
// Go: ast/utilities.go:362 IsOptionalChainRoot

// Determines whether a node is the outermost `OptionalChain` in an ECMAScript `OptionalExpression`:
//
//  1. For `a?.b.c`, the outermost chain is `a?.b.c` (`c` is the end of the chain starting at `a?.`)
//  2. For `a?.b!`, the outermost chain is `a?.b` (`b` is the end of the chain starting at `a?.`)
//  3. For `(a?.b.c).d`, the outermost chain is `a?.b.c` (`c` is the end of the chain starting at `a?.` since parens end the chain)
//  4. For `a?.b.c?.d`, both `a?.b.c` and `a?.b.c?.d` are outermost (`c` is the end of the chain starting at `a?.`, and `d` is
//     the end of the chain starting at `c?.`)
//  5. For `a?.(b?.c).d`, both `b?.c` and `a?.(b?.c)d` are outermost (`c` is the end of the chain starting at `b`, and `d` is
//     the end of the chain starting at `a?.`)
// Go: ast/utilities.go:375 IsOutermostOptionalChain

// Determines whether a node is the expression preceding an optional chain (i.e. `a` in `a?.b`).
// Go: ast/utilities.go:383 IsExpressionOfOptionalChainRoot

// Go: ast/utilities.go:387 IsNullishCoalesce

// Go: ast/utilities.go:391 IsAssertionExpression

// Go: ast/utilities.go:396 isLeftHandSideExpressionKind
fn is_left_hand_side_expression_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::NewExpression
            | SyntaxKind::CallExpression
            | SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment
            | SyntaxKind::TaggedTemplateExpression
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ClassExpression
            | SyntaxKind::FunctionExpression
            | SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateExpression
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::ThisKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::NonNullExpression
            | SyntaxKind::ExpressionWithTypeArguments
            | SyntaxKind::MetaProperty
            | SyntaxKind::ImportKeyword
            | SyntaxKind::MissingDeclaration
    )
}

// Determines whether a node is a LeftHandSideExpression based only on its kind.
// Go: ast/utilities.go:411 IsLeftHandSideExpression
// PERF: binderview1. The kind is read once. Only a PartiallyEmittedExpression
// is an outer expression of `OEK_PARTIALLY_EMITTED_EXPRESSIONS`, so any other
// node is its own `SkipPartiallyEmittedExpressions`.

// Go: ast/utilities.go:415 isUnaryExpressionKind
fn is_unary_expression_kind(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::PrefixUnaryExpression
        | SyntaxKind::PostfixUnaryExpression
        | SyntaxKind::DeleteExpression
        | SyntaxKind::TypeOfExpression
        | SyntaxKind::VoidExpression
        | SyntaxKind::AwaitExpression
        | SyntaxKind::TypeAssertionExpression => true,
        _ => is_left_hand_side_expression_kind(kind),
    }
}

// Determines whether a node is a UnaryExpression based only on its kind.
// Go: ast/utilities.go:430 IsUnaryExpression

// Go: ast/utilities.go:434 isExpressionKind
fn is_expression_kind(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::ConditionalExpression
        | SyntaxKind::YieldExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::BinaryExpression
        | SyntaxKind::SpreadElement
        | SyntaxKind::AsExpression
        | SyntaxKind::OmittedExpression
        | SyntaxKind::PartiallyEmittedExpression
        | SyntaxKind::SatisfiesExpression => true,
        _ => is_unary_expression_kind(kind),
    }
}

// Determines whether a node is an expression based only on its kind.
// Go: ast/utilities.go:451 IsExpression

// Go: ast/utilities.go:455 IsCommaExpression

// Go: ast/utilities.go:459 IsCommaSequence

// Go: ast/utilities.go:463 IsIterationStatement

// Determines if a node is a property or element access expression
// Go: ast/utilities.go:479 IsAccessExpression
// PERF: binderview1. The kind is read once: a kind read is a store lookup
// that the compiler does not merge with a second one.

/// A fixed set of `SyntaxKind`s, one bit per kind, built at compile time.
// PERF: the hot kind tests below (`is_statement`, `is_function_like`,
// `is_private_identifier_class_element_declaration`) are one table load and
// one bit test, not a chain of compares. The answers are the same as the Go
// `switch` statements, whose kind lists are kept here in Go order.
#[derive(Clone, Copy)]
struct KindSet([u64; 8]);

// `KindSet::contains` masks the word index with `& 7`, so 8 words (512 bits)
// must cover every kind. The mask removes the bounds check.
const _: () = assert!(SyntaxKind::COUNT <= 512);

impl KindSet {

    const EMPTY: Self = Self([0; 8]);

    /// This set plus `kinds`.
    const fn with(mut self, kinds: &[SyntaxKind]) -> Self {
        let mut i = 0;
        while i < kinds.len() {
            let k = kinds[i] as usize;
            self.0[k >> 6] |= 1 << (k & 63);
            i += 1;
        }
        self
    }

    #[inline]
    const fn contains(&self, kind: SyntaxKind) -> bool {
        let k = kind as usize;
        self.0[(k >> 6) & 7] & (1 << (k & 63)) != 0
    }
}

// The kinds of Go `isFunctionLikeDeclarationKind` (ast/utilities.go:483).
const FUNCTION_LIKE_DECLARATION_KIND_LIST: &[SyntaxKind] = &[
    SyntaxKind::FunctionDeclaration,
    SyntaxKind::MethodDeclaration,
    SyntaxKind::Constructor,
    SyntaxKind::GetAccessor,
    SyntaxKind::SetAccessor,
    SyntaxKind::FunctionExpression,
    SyntaxKind::ArrowFunction,
];


// The kinds of Go `IsFunctionLikeKind` (ast/utilities.go:503): these kinds,
// then the function-like declaration kinds.

// Go: ast/utilities.go:483 isFunctionLikeDeclarationKind

// Determines if a node is function-like (but is not a signature declaration)
// Go: ast/utilities.go:498 IsFunctionLikeDeclaration

// Go: ast/utilities.go:503 IsFunctionLikeKind

// Determines if a node is function- or signature-like.
// Go: ast/utilities.go:518 IsFunctionLike

// Go: ast/utilities.go:523 IsFunctionLikeOrClassStaticBlockDeclaration

// Go: ast/utilities.go:527 IsFunctionOrSourceFile

// Go: ast/utilities.go:531 IsClassLike

// Go: ast/utilities.go:535 IsClassOrInterfaceLike

// Go: ast/utilities.go:539 IsClassElement

// Go: ast/utilities.go:554 IsMethodOrAccessor

// The kinds of `IsPropertyDeclaration || IsMethodOrAccessor` in Go
// `IsPrivateIdentifierClassElementDeclaration` (ast/utilities.go:562).

// Go: ast/utilities.go:562 IsPrivateIdentifierClassElementDeclaration

// Go: ast/utilities.go:566 IsObjectLiteralOrClassExpressionMethodOrAccessor

// Go: ast/utilities.go:572 IsTypeElement

// Go: ast/utilities.go:587 IsObjectLiteralElement

// Go: ast/utilities.go:600 IsObjectLiteralMethod

// Go: ast/utilities.go:604 IsAutoAccessorPropertyDeclaration

// Go: ast/utilities.go:608 IsParameterPropertyDeclaration

// Go: ast/utilities.go:612 IsJsxChild

// Go: ast/utilities.go:624 IsJsxAttributeLike

// The kinds of Go `isDeclarationStatementKind` (ast/utilities.go:628).
const DECLARATION_STATEMENT_KIND_LIST: &[SyntaxKind] = &[
    SyntaxKind::FunctionDeclaration,
    SyntaxKind::MissingDeclaration,
    SyntaxKind::ClassDeclaration,
    SyntaxKind::InterfaceDeclaration,
    SyntaxKind::TypeAliasDeclaration,
    SyntaxKind::JsTypeAliasDeclaration,
    SyntaxKind::EnumDeclaration,
    SyntaxKind::ModuleDeclaration,
    SyntaxKind::ImportDeclaration,
    SyntaxKind::JsImportDeclaration,
    SyntaxKind::ImportEqualsDeclaration,
    SyntaxKind::ExportDeclaration,
    SyntaxKind::ExportAssignment,
    SyntaxKind::NamespaceExportDeclaration,
];


// Go: ast/utilities.go:628 isDeclarationStatementKind

// Determines whether a node is a DeclarationStatement. Ideally this does not use Parent pointers, but it may use them
// to rule out a Block node that is part of `try` or `catch` or is the Block-like body of a function.
//
// NOTE: ECMA262 would just call this a Declaration
// Go: ast/utilities.go:653 IsDeclarationStatement

// The kinds of Go `isStatementKindButNotDeclarationKind` (ast/utilities.go:657).
const STATEMENT_BUT_NOT_DECLARATION_KIND_LIST: &[SyntaxKind] = &[
    SyntaxKind::BreakStatement,
    SyntaxKind::ContinueStatement,
    SyntaxKind::DebuggerStatement,
    SyntaxKind::DoStatement,
    SyntaxKind::ExpressionStatement,
    SyntaxKind::EmptyStatement,
    SyntaxKind::ForInStatement,
    SyntaxKind::ForOfStatement,
    SyntaxKind::ForStatement,
    SyntaxKind::IfStatement,
    SyntaxKind::LabeledStatement,
    SyntaxKind::ReturnStatement,
    SyntaxKind::SwitchStatement,
    SyntaxKind::ThrowStatement,
    SyntaxKind::TryStatement,
    SyntaxKind::VariableStatement,
    SyntaxKind::WhileStatement,
    SyntaxKind::WithStatement,
    SyntaxKind::NotEmittedStatement,
];


// The two statement kind lists together, for `is_statement`.

// Go: ast/utilities.go:657 isStatementKindButNotDeclarationKind

// Determines whether a node is a Statement that is not also a Declaration. Ideally this does not use Parent pointers,
// but it may use them to rule out a Block node that is part of `try` or `catch` or is the Block-like body of a function.
//
// NOTE: ECMA262 would just call this a Statement
// Go: ast/utilities.go:687 IsStatementButNotDeclaration

// Determines whether a node is a Statement. Ideally this does not use Parent pointers, but it may use
// them to rule out a Block node that is part of `try` or `catch` or is the Block-like body of a function.
//
// NOTE: ECMA262 would call this either a StatementListItem or ModuleListItem
// Go: ast/utilities.go:695 IsStatement

// Determines whether a node is a BlockStatement. If parents are available, this ensures the Block is
// not part of a `try` statement, `catch` clause, or the Block-like body of a function
// Go: ast/utilities.go:702 isBlockStatement

// Determines whether a node is the Block-like body of a function by walking the parent of the node
// Go: ast/utilities.go:713 IsFunctionBlock

// Go: ast/utilities.go:717 IsBlockOrCatchScoped
// PERF: lsshells M3 repair. `BLOCK_SCOPED` has no binder bit, so the walk
// reads no binder data (`get_combined_parser_flags`).

// Go: ast/utilities.go:721 IsCatchClauseVariableDeclarationOrBindingElement

// Go: ast/utilities.go:726 IsTypeNodeKind
pub fn is_type_node_kind(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AnyKeyword
        | SyntaxKind::UnknownKeyword
        | SyntaxKind::NumberKeyword
        | SyntaxKind::BigIntKeyword
        | SyntaxKind::ObjectKeyword
        | SyntaxKind::BooleanKeyword
        | SyntaxKind::StringKeyword
        | SyntaxKind::SymbolKeyword
        | SyntaxKind::VoidKeyword
        | SyntaxKind::UndefinedKeyword
        | SyntaxKind::NeverKeyword
        | SyntaxKind::IntrinsicKeyword
        | SyntaxKind::ExpressionWithTypeArguments
        | SyntaxKind::JsDocAllType
        | SyntaxKind::JsDocNullableType
        | SyntaxKind::JsDocNonNullableType
        | SyntaxKind::JsDocOptionalType
        | SyntaxKind::JsDocVariadicType => return true,
        _ => {}
    }
    (kind as u16) >= (SyntaxKind::FIRST_TYPE_NODE as u16)
        && (kind as u16) <= (SyntaxKind::LAST_TYPE_NODE as u16)
}

// Go: ast/utilities.go:751 IsTypeNode

// Go: ast/utilities.go:755 IsJSDocKind
pub fn is_js_doc_kind(kind: SyntaxKind) -> bool {
    (SyntaxKind::FIRST_JS_DOC_NODE as u16) <= (kind as u16)
        && (kind as u16) <= (SyntaxKind::LAST_JS_DOC_NODE as u16)
}

// Go: ast/utilities.go:759 IsJSDocTypeAssertion

// Go: ast/utilities.go:767 IsPrologueDirective

// PORT: `crate::flags` defines the single-bit `OEK*` consts. The composite
// consts from the same Go const block are defined here.
impl OuterExpressionKinds {
    pub const OEK_ASSERTIONS: Self =
        Self(Self::OEK_TYPE_ASSERTIONS.0 | Self::OEK_NON_NULL_ASSERTIONS.0 | Self::OEK_SATISFIES.0);
    pub const OEK_ALL: Self = Self(
        Self::OEK_PARENTHESES.0
            | Self::OEK_ASSERTIONS.0
            | Self::OEK_PARTIALLY_EMITTED_EXPRESSIONS.0
            | Self::OEK_EXPRESSIONS_WITH_TYPE_ARGUMENTS.0,
    );
    pub const OEK_ALL_EXCEPT_ASSERTIONS_OR_EXPRESSIONS_WITH_TYPE_ARGUMENTS: Self = Self(
        Self::OEK_ALL.0 & !Self::OEK_ASSERTIONS.0 & !Self::OEK_EXPRESSIONS_WITH_TYPE_ARGUMENTS.0,
    );
    pub const OEK_EXPRESSION_TYPE_PASSTHROUGH: Self =
        Self(Self::OEK_PARENTHESES.0 | Self::OEK_ASSIGNMENTS.0 | Self::OEK_COMMA.0);
}

// Determines whether node is an "outer expression" of the provided kinds
// Go: ast/utilities.go:791 IsOuterExpression

// Descends into an expression, skipping past "outer expressions" of the provided kinds
// Go: ast/utilities.go:817 SkipOuterExpressions

// Skips past the parentheses of an expression
// Go: ast/utilities.go:829 SkipParentheses

// Go: ast/utilities.go:833 SkipTypeParentheses

// Go: ast/utilities.go:840 SkipPartiallyEmittedExpressions

// Walks up the parents of a parenthesized expression to find the containing node
// Go: ast/utilities.go:845 WalkUpParenthesizedExpressions

// Walks up the parents of a parenthesized type to find the containing node
// Go: ast/utilities.go:853 WalkUpParenthesizedTypes

// Walks up the parents of a node to find the containing SourceFile
// Go: ast/utilities.go:861 GetSourceFileOfNode
// PORT: a frozen store node whose parent walk ends at its store root gets
// that root in O(1) (`ast::store::frozen_source_file_of_node`). Other nodes
// (synthetic nodes, nodes before the freeze) walk the parents like Go.


// Go: ast/utilities.go:877 newParentInChildrenSetter
// PORT: Go keeps `parent` in a closure state and recurses through
// `node.ForEachChild(state.visit)`. Rust carries the same state in a
// struct. Go pools the closure with `sync.Pool`; that is only an
// allocation cache, so it is not ported.
struct ParentInChildrenSetter {
    parent: Node,
}

impl ParentInChildrenSetter {
}

fn new_parent_in_children_setter() -> ParentInChildrenSetter {
    ParentInChildrenSetter { parent: Node::NIL }
}

// Go: ast/utilities.go:899 SetParentInChildren

#[cfg(test)]
mod symbol_id_tests {
    use super::*;

    /// Two checker arenas made from one binder arena share the ids of the
    /// binder symbols. Their own symbols (`is_own_index`) get ids of their
    /// own, also at one index in both arenas, and no later bind uses their
    /// index, so a checker that catches up reads the later binder symbol
    /// with its shared id.
    #[test]
    fn checker_symbols_have_ids_of_their_own() {
        let mut binder = SymbolArena::new();
        let shared = binder.new_symbol(SymbolFlags::NONE, "shared");
        let mut first = binder.for_checker();
        // A later program binds more files into the same binder arena.
        let later = binder.new_symbol(SymbolFlags::NONE, "later");
        let mut second = binder.for_checker();
        let own_first = first.new_symbol(SymbolFlags::NONE, "own");
        let own_second = second.new_symbol(SymbolFlags::NONE, "own");
        assert_eq!(
            own_first, own_second,
            "the test needs one index for two symbols"
        );
        assert!(is_own_index(own_first.0) && !is_own_index(later.0));

        let id_later = get_symbol_id(&second, later);
        let id_own_first = get_symbol_id(&first, own_first);
        let id_own_second = get_symbol_id(&second, own_second);
        assert_ne!(id_own_first, id_later);
        assert_ne!(id_own_first, id_own_second);
        assert_ne!(id_own_second, id_later);
        assert_eq!(get_symbol_id(&binder, later), id_later);
        assert_eq!(
            get_symbol_id(&first, shared),
            get_symbol_id(&second, shared)
        );
        // An id stays when the thread switches between arenas.
        assert_eq!(get_symbol_id(&first, own_first), id_own_first);
        assert_eq!(get_symbol_id(&second, own_second), id_own_second);
        // `first` catches up to the later bind.
        assert_eq!(first.symbol_at_slot(second.id_slot(later)), None);
        first.catch_up(&binder, &[]);
        assert_eq!(first.symbol_at_slot(second.id_slot(later)), Some(later));
        assert_eq!(first.sym(later).name.as_str(), "later");
        assert_eq!(get_symbol_id(&first, later), id_later);
        assert_eq!(get_symbol_id(&first, own_first), id_own_first);

        // A dropped checker arena frees its ids on this thread.
        let key = first.id_slot(own_first).key;
        assert_ne!(key, 0, "a checker symbol has an id of its own");
        drop(first);
        OWN_SYMBOL_IDS.with(|own| {
            let own = own.borrow();
            assert!(own.key != key && !own.others.contains_key(&key));
        });
    }

    /// A shadow (`SymbolArena::push_shadow`) is the same Go symbol as its
    /// origin: one id, given on first use through either, and kept after
    /// the arena of the origin drops.
    #[test]
    fn shadows_have_the_id_of_their_origin() {
        let mut binder = SymbolArena::new();
        let mut first = binder.for_checker();
        // A later program binds a symbol that `first` does not have.
        let later = binder.new_symbol(SymbolFlags::NONE, "later");
        let mut second = binder.for_checker();
        let own = first.new_symbol(SymbolFlags::TRANSIENT, "own");

        let origin = first.id_slot(own);
        let shadow = second.push_shadow(first.sym(own).clone(), origin);
        assert_eq!(second.symbol_at_slot(origin), Some(shadow));
        assert_eq!(first.symbol_at_slot(second.id_slot(shadow)), Some(own));
        let id = get_symbol_id(&second, shadow);
        assert_eq!(get_symbol_id(&first, own), id);

        let lineage = second.id_slot(later);
        assert_eq!(lineage.key, 0, "a binder symbol has the shared slot");
        assert_eq!(first.symbol_at_slot(lineage), None);
        let shadow_later = first.push_shadow(second.sym(later).clone(), lineage);
        assert_eq!(
            get_symbol_id(&first, shadow_later),
            get_symbol_id(&binder, later)
        );

        drop(first);
        assert_eq!(get_symbol_id(&second, shadow), id);
    }

    /// An id chunk of `LineageIds` holds the symbols of one `SymbolArena`
    /// chunk, so a freed lineage range frees whole id chunks.
    #[test]
    fn lineage_id_chunks_are_symbol_chunks() {
        let mut arena = SymbolArena::new_file();
        arena.new_symbol(SymbolFlags::NONE, "a");
        arena.end_chunk();
        assert_eq!(arena.symbol_count(), LINEAGE_ID_CHUNK);
    }

    /// When the lineage frees the symbols of a dead file version, a thread
    /// frees its id chunks of them when it next makes a chunk. The other
    /// ids stay, and an id read of a freed symbol panics.
    #[test]
    #[should_panic(expected = "file version 4194297 is released")]
    fn freed_lineage_symbols_free_their_ids() {
        const FILE: usize = (1 << 22) - 7;
        // Far above the indexes that other tests bind: every thread of the
        // process frees this range.
        const START: usize = 1 << 26;
        let mut ids = LineageIds::new();
        let kept = ids.id(START - 1);
        let dying = ids.id(START + 5);
        assert_eq!(ids.id(START + 5), dying);

        free_lineage_symbol_ids(FILE, START..START + 2 * LINEAGE_ID_CHUNK);
        // Until the next new chunk the id stays.
        assert_eq!(ids.id(START + 5), dying);
        let later = ids.id(START + 2 * LINEAGE_ID_CHUNK);
        assert!(later > dying);
        assert!(ids.chunks[START >> LINEAGE_ID_SHIFT].is_none());
        assert_eq!(ids.id(START - 1), kept);
        ids.id(START + LINEAGE_ID_CHUNK + 1);
    }
}

#[cfg(test)]
mod binary_view_tests {
    use super::*;

    /// binderview1: a `BinaryView` holds what the field accessors read:
    /// before the publish (the accessor path), after it in an alias-free
    /// store (`FrozenIds::Direct`) and in a store with an alias slot (a
    /// synthetic right operand, `FrozenIds::Records`), and with no ids. It
    /// publishes, so no other test may build or publish stores while it runs
    /// (the runner uses one thread).
    #[test]
    fn binary_view_reads_what_the_accessors_read() {
        let plain = new_file_store("/binaryview/a.ts", "x = y;");
        let aliased = new_file_store("/binaryview/b.ts", "x + s;");
        let make = |file: usize, operator: SyntaxKind, right: Node| {
            let f = NodeFactory::for_file(file);
            let left = f.new_identifier("x");
            let operator_token = f.new_token(operator);
            f.new_binary_expression(ModifierList::NIL, left, Node::NIL, operator_token, right)
        };
        let a = make(
            plain,
            SyntaxKind::EqualsToken,
            NodeFactory::for_file(plain).new_identifier("y"),
        );
        let b = make(
            aliased,
            SyntaxKind::PlusToken,
            NodeFactory::new().new_identifier("s"),
        );
        let check = |when: &str| {
            for n in [a, b] {
                let operator_token = n.operator_token();
                let read = BinaryView {
                    left: n.left(),
                    operator_token,
                    operator: operator_token.kind(),
                    right: n.right(),
                    type_: n.type_(),
                };
                assert_eq!(BinaryView::new(n), read, "{when}");
                assert_eq!(BinaryView::with_ids(n, None), read, "{when}");
            }
        };
        check("built");
        freeze_file_store(plain);
        freeze_file_store(aliased);
        crate::program::publish_parsed_files("/");
        assert!(frozen_store_ast_node(a).is_some(), "a is published");
        assert!(matches!(
            frozen_store_ids(plain),
            Some(FrozenIds::Direct { .. })
        ));
        assert!(matches!(
            frozen_store_ids(aliased),
            Some(FrozenIds::Records { .. })
        ));
        check("published");
        assert_eq!(BinaryView::new(a).operator, SyntaxKind::EqualsToken);
        assert_eq!(BinaryView::new(b).operator, SyntaxKind::PlusToken);
    }
}
