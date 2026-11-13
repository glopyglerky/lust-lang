//! Port of Go `api/encoder/encoder.go`: the binary AST encoder.
//!
//! PORT notes for the whole file:
//! - Go `int` byte lengths and offsets are `usize` (they index byte buffers).
//! - Go `appendUint32s` and the msgpack writers return the grown slice; here
//!   they append to a `&mut Vec<u8>` in place.
//! - Go `n := node.AsX()` type assertions are the per-field `Node` accessors,
//!   which panic on other kinds like the Go assertion.
//! - Go `sourceFile.Hash`, `ParseOptions()`, `NodeCount` and `TextCount` are
//!   not on the ts_goport SourceFile node. They are in its `ParsedSourceFile`.
//!   A caller that holds that record gives it (`encode_parsed_source_file`,
//!   the leased file of an api `createSourceFile`). Otherwise the encoder
//!   finds it by the root node: see `source_file_content_hash` and
//!   `parsed_source_file_of`.
//! - The other Go `SourceFile` fields that the parser sets (`Path()`,
//!   `Imports()`, `ModuleAugmentations`, ...) are read through
//!   `source_file_fields`: from the record that the caller gives, else from
//!   the published file, else from the parse of a file that is parsed
//!   outside a program.

use crate::api::encoder::prelude::*;

use crate::ast::node::{ChildField, };
use crate::ast::source_file_ls::{SourceFileDataKey, new_source_file_data_key};
use crate::ast::store::{};
use crate::astdata::NodeData;
use crate::frontend::core_binarysearch::binary_search_unique_func;
use crate::frontend::parser::{ExternalModuleIndicatorOptions, };
use std::borrow::Cow;
use std::cell::OnceCell;
use std::sync::LazyLock;

// Go: api/encoder/encoder.go:16 init
// PORT: the Go `init()` guard is a compile-time assertion.
const _: () = assert!(
    (SyntaxKind::LAST_UNARY_OPERATOR as u32) <= 0x3f,
    "KindLastUnaryOperator exceeds the 6-bit commonData capacity (max 63)"
);

// Go: api/encoder/encoder.go:21 NodeOffsetKind ... NodeSize
pub const NODE_OFFSET_KIND: usize = 0;
pub const NODE_OFFSET_POS: usize = 4;
pub const NODE_OFFSET_END: usize = 8;
pub const NODE_OFFSET_NEXT: usize = 12;
pub const NODE_OFFSET_PARENT: usize = 16;
pub const NODE_OFFSET_DATA: usize = 20;
pub const NODE_OFFSET_FLAGS: usize = 24;
/// NodeSize is the number of bytes that represents a single node in the encoded format.
pub const NODE_SIZE: usize = 28;

// Go: api/encoder/encoder.go:33 NodeDataTypeChildren ...
pub const NODE_DATA_TYPE_CHILDREN: u32 = 0 << 30;
pub const NODE_DATA_TYPE_STRING: u32 = 1 << 30;
pub const NODE_DATA_TYPE_EXTENDED_DATA: u32 = 2 << 30;

// Go: api/encoder/encoder.go:39 NodeDataTypeMask ...
pub const NODE_DATA_TYPE_MASK: u32 = 0xc0_00_00_00;
pub const NODE_DATA_CHILD_MASK: u32 = 0x00_00_00_ff;
pub const NODE_DATA_STRING_INDEX_MASK: u32 = 0x00_ff_ff_ff;

// Go: api/encoder/encoder.go:45 SyntaxKindNodeList
pub const SYNTAX_KIND_NODE_LIST: u32 = u32::MAX;

// Go: api/encoder/encoder.go:49 HeaderOffsetMetadata ... HeaderSize
pub const HEADER_OFFSET_METADATA: usize = 0;
pub const HEADER_OFFSET_HASH_LO0: usize = 4;
pub const HEADER_OFFSET_HASH_LO1: usize = 8;
pub const HEADER_OFFSET_HASH_HI0: usize = 12;
pub const HEADER_OFFSET_HASH_HI1: usize = 16;
pub const HEADER_OFFSET_PARSE_OPTIONS: usize = 20;
pub const HEADER_OFFSET_STRING_OFFSETS: usize = 24;
pub const HEADER_OFFSET_STRING_DATA: usize = 28;
pub const HEADER_OFFSET_EXTENDED_DATA: usize = 32;
pub const HEADER_OFFSET_STRUCTURED_DATA: usize = 36;
pub const HEADER_OFFSET_NODES: usize = 40;
// ts#64434: source file ID and lease ID (uint64 each) and the binder data
// offset.
pub const HEADER_OFFSET_SOURCE_FILE_ID: usize = 44;
pub const HEADER_OFFSET_SOURCE_FILE_LEASE: usize = 52;
pub const HEADER_OFFSET_BINDER_DATA: usize = 60;
pub const HEADER_SIZE: usize = 64;

// Go: api/encoder/encoder.go:71 ProtocolVersion
// ts#63957: 7 -> 8. ts#64434: 8 -> 9.
pub const PROTOCOL_VERSION: u8 = 9;

// Source File Binary Format
// =========================
//
// The following defines a protocol for serializing TypeScript SourceFile objects to a compact binary format. All integer
// values are little-endian.
//
// Overview
// --------
//
// The format comprises seven sections:
//
// | Section            | Length             | Description                                                                                     |
// | ------------------ | ------------------ | ----------------------------------------------------------------------------------------------- |
// | Header             | 64 bytes           | Contains the content hash, parse options, flags, file metadata, and byte offsets to the start of each section. |
// | String offsets     | 8 bytes per string | Pairs of starting byte offsets and ending byte offsets into the **string data** section.        |
// | String data        | variable           | UTF-8 encoded string data.                                                                      |
// | Extended node data | variable           | Extra data for some kinds of nodes.                                                             |
// | Structured data    | variable           | Msgpack-encoded metadata blobs (e.g. file references).                                         |
// | Nodes              | 28 bytes per node  | Defines the AST structure of the file, with references to strings and extended data.            |
//
// Header (64 bytes)
// -----------------
//
// The header contains the following fields:
//
// | Byte offset | Type      | Field                                             |
// | ----------- | --------- | ------------------------------------------------- |
// | 0           | uint8     | Protocol version                                  |
// | 1-3         |           | Reserved                                          |
// | 4-19        | uint128   | Source file content hash (xxh3, LE)               |
// | 20-23       | uint32    | Parse options (bitmask; bit 0: JSX, bit 1: Force) |
// | 24-27       | uint32    | Byte offset to string offsets section             |
// | 28-31       | uint32    | Byte offset to string data section                |
// | 32-35       | uint32    | Byte offset to extended node data section         |
// | 36-39       | uint32    | Byte offset to structured data section            |
// | 40-43       | uint32    | Byte offset to nodes section                      |
// | 44-51       | uint64    | Source file ID (0 = none)                          |
// | 52-59       | uint64    | Source file lease ID (0 = none)                    |
// | 60-63       | uint32    | Byte offset to binder data (0 = none)              |
//
// String offsets (8 bytes per string)
// -----------------------------------
//
// Each string offset entry consists of two 4-byte unsigned integers, representing the start and end byte offsets into the
// **string data** section.
//
// String data (variable)
// ----------------------
//
// The string data section contains UTF-8 encoded string data, with WTF-8 used for JS strings containing lone UTF-16
// surrogates. In typical cases, the entirety of the string data is the source file text, and individual nodes with
// string properties reference their positional slice of the file text. In cases where a node's string property is not
// equal to the slice of file text at its position, the unique string is appended to the string data section after the
// file text.
//
// Extended node data (variable)
// -----------------------------
//
// The extended node data section contains additional data for specific node types. The length and meaning of each entry
// is defined by the node type.
//
// Currently, the only node types that use this section are `TemplateHead`, `TemplateMiddle`, `TemplateTail`, and
// `SourceFile`. The extended data format for the first three is:
//
// | Byte offset | Type   | Field                                            |
// | ----------- | ------ | ------------------------------------------------ |
// | 0-4         | uint32 | Index of `text` in the string offsets section    |
// | 4-8         | uint32 | Index of `rawText` in the string offsets section |
// | 8-12        | uint32 | Value of `templateFlags`                         |
//
// and for `SourceFile` is:
//
// | Byte offset | Type   | Field                                                          |
// | ----------- | ------ | -------------------------------------------------------------- |
// | 0-4         | uint32 | Index of `text` in the string offsets section                  |
// | 4-8         | uint32 | Index of `fileName` in the string offsets section              |
// | 8-12        | uint32 | Index of `path` in the string offsets section                  |
// | 12-16       | uint32 | Value of `languageVariant`                                    |
// | 16-20       | uint32 | Value of `scriptKind`                                         |
// | 20-24       | uint32 | Byte offset of `referencedFiles` in structured data section   |
// | 24-28       | uint32 | Byte offset of `typeReferenceDirectives` in structured data   |
// | 28-32       | uint32 | Byte offset of `libReferenceDirectives` in structured data    |
// | 32-36       | uint32 | Byte offset of `imports` node index array in structured data  |
// | 36-40       | uint32 | Byte offset of `moduleAugmentations` node index array         |
// | 40-44       | uint32 | Byte offset of `ambientModuleNames` string array              |
// | 44-48       | uint32 | Node index of `externalModuleIndicator` (0 = nil)             |
// | 48-52       | uint32 | Index of `originalText` in the string offsets section         |
// | 52-56       | uint32 | Byte offset of `spanMap` in structured data                    |
// | 56-60       | uint32 | Byte offset of `supplementalSourceFileNames` in structured data |
// | 60-64       | uint32 | Index of `canonicalSourceFileName`, or noStructuredData         |
// | 64-68       | uint32 | Index of `contentMapper`, or noStructuredData                   |
// | 68-72       | uint32 | Index of `virtualFileName`, or noStructuredData                 |
// | 72-76       | uint32 | Byte offset of `diagnosticDirectives` in structured data       |
//
// Structured data (variable)
// --------------------------
//
// The structured data section contains msgpack-encoded metadata blobs. Each blob is a self-contained
// msgpack value. File reference arrays use the following tuple format:
//
//   [pos: uint, end: uint, fileName: string, resolutionMode: uint, preserve: bool]
//
// Node index arrays (imports, moduleAugmentations) are msgpack arrays of uint values, where each
// value is a node index into the nodes section. String arrays (ambientModuleNames) are msgpack
// arrays of string values.
//
// Span maps are msgpack arrays of tuples in UTF-16 coordinates:
//
//   [virtualStart: uint, virtualLength: uint, originalStart: uint, originalLength: uint, kind: uint, features?: uint]
//
// Diagnostic directives are msgpack arrays of normalized tuples in UTF-16 coordinates:
//
//   [originalStart: uint, originalLength: uint, virtualStart: uint, virtualLength: uint, policy: uint, unusedCode: uint]
//
// An offset of 0xFFFFFFFF indicates no data (empty array).
//
// Nodes (28 bytes per node)
// -------------------------
//
// The nodes section contains the AST structure of the file. Nodes are represented in a flat array in source order,
// heavily inspired by https://marvinh.dev/blog/speeding-up-javascript-ecosystem-part-11/. Each node has the following
// structure:
//
// | Byte offset | Type   | Field                      |
// | ----------- | ------ | -------------------------- |
// | 0-4         | uint32 | Kind                       |
// | 4-8         | uint32 | Pos                        |
// | 8-12        | uint32 | End                        |
// | 12-16       | uint32 | Node index of next sibling |
// | 16-20       | uint32 | Node index of parent       |
// | 20-24       |        | Node data                  |
// | 24-28       | uint32 | Node flags                 |
//
// The first 28 bytes of the nodes section are zeros representing a nil node, such that nodes without a parent or next
// sibling can unambiuously use `0` for those indices.
//
// NodeLists are represented as normal nodes with the special `kind` value `0xff_ff_ff_ff`. They are considered the parent
// of their contents in the encoded format. A client reconstructing an AST similar to TypeScript's internal representation
// should instead set the `parent` pointers of a NodeList's children to the NodeList's parent. A NodeList's `data` field
// is the uint32 length of the list, and does not use one of the data types described below. A NodeList's `flags` field
// is not used for AST node flags (NodeLists have none); bit 0 instead encodes `HasTrailingComma`.
//
// For node types other than NodeList, the node data field encodes one of the following, determined by the first 2 bits of
// the field:
//
// | Value | Data type | Description                                                                          |
// | ----- | --------- | ------------------------------------------------------------------------------------ |
// | 0b00  | Children  | Disambiguates which named properties of the node its children should be assigned to. |
// | 0b01  | String    | The index of the node's string property in the **string offsets** section.           |
// | 0b10  | Extended  | The byte offset of the node's extended data into the **extended node data** section. |
// | 0b11  | Reserved  | Reserved for future use.                                                             |
//
// In all node data types, the remaining 6 bits of the first byte are used to encode small values specific to the node
// type. For most node types, these are individual boolean flags. For unary expressions, all 6 bits encode the operator's
// SyntaxKind value (e.g., PlusPlusToken=45, TildeToken=54), which fits because KindLastUnaryOperator (54) <= 0x3f (63).
//
// | Node type                    | Bits 0-5                              | Notes                          |
// | ---------------------------- | ------------------------------------- | ------------------------------ |
// | `ImportSpecifier`            | Bit 0: `isTypeOnly`                   |                                |
// | `ImportClause`               | Bit 0: `isTypeOnly`, Bit 1: `isDefer` |                                |
// | `ExportSpecifier`            | Bit 0: `isTypeOnly`                   |                                |
// | `ImportEqualsDeclaration`    | Bit 0: `isTypeOnly`                   |                                |
// | `ExportDeclaration`          | Bit 0: `isTypeOnly`                   |                                |
// | `ImportTypeNode`             | Bit 0: `isTypeOf`                     |                                |
// | `ExportAssignment`           | Bit 0: `isExportEquals`               |                                |
// | `Block`                      | Bit 0: `multiline`                    |                                |
// | `ArrayLiteralExpression`     | Bit 0: `multiline`                    |                                |
// | `ObjectLiteralExpression`    | Bit 0: `multiline`                    |                                |
// | `JsxText`                    | Bit 0: `containsOnlyTriviaWhiteSpaces`|                                |
// | `JSDocTypeLiteral`           | Bit 0: `isArrayType`                  |                                |
// | `JsDocPropertyTag`           | Bit 0: `isBracketed`, Bit 1: `isNameFirst` |                           |
// | `JsDocParameterTag`          | Bit 0: `isBracketed`, Bit 1: `isNameFirst` |                           |
// | `VariableDeclarationList`    | Bit 0: is `let`, Bit 1: is `const`    |                                |
// | `ImportAttributes`           | Bit 0: `multiline`, Bit 1: is `assert`|                                |
// | `PrefixUnaryExpression`      | Bits 0-5: operator SyntaxKind         | e.g., `!`, `~`, `++`, `--`    |
// | `PostfixUnaryExpression`     | Bits 0-5: operator SyntaxKind         | e.g., `++`, `--`               |
//
// The remaining 3 bytes of the node data field vary by data type:
//
// ### Children (0b00)
//
// If a node has fewer children than its type allows, additional data is needed to determine which properties the children
// correspond to. The last byte of the 4-byte data field is a bitmask representing the child properties of the node type,
// in visitor order, where `1` indicates that the child at that property is present and `0` indicates that the property is
// nil. For example, a `MethodDeclaration` has the following child properties:
//
// | Property name  | Bit position |
// | -------------- | ------------ |
// | modifiers      | 0            |
// | asteriskToken  | 1            |
// | name           | 2            |
// | postfixToken   | 3            |
// | typeParameters | 4            |
// | parameters     | 5            |
// | returnType     | 6            |
// | body           | 7            |
//
// A bitmask with value `0b01100101` would indicate that the next four direct descendants (i.e., node records that have a
// `parent` set to the node index of the `MethodDeclaration`) of the node are its `modifiers`, `name`, `parameters`, and
// `body` properties, in that order. The remaining properties are nil. (To reconstruct the node with named properties, the
// client must consult a static table of each node type's child property names.)
//
// The bitmask may be zero for node types that can only have a single child, since no disambiguation is needed.
// Additionally, the children data type may be used for nodes that can never have children, but do not require other
// data types.
//
// ### String (0b01)
//
// The string data type is used for nodes with a single string property. (Currently, the name of that property is always
// `text`.) The last three bytes of the 4-byte data field form a single 24-bit unsigned integer (i.e.,
// `uint32(0x00_ff_ff_ff & node.data)`) _N_ that is an index into the **string offsets** section. The *N*th 32-bit
// unsigned integer in the **string offsets** section is the byte offset of the start of the string in the **string data**
// section, and the *N+1*th 32-bit unsigned integer is the byte offset of the end of the string in the
// **string data** section.
//
// ### Extended (0b10)
//
// The extended data type is used for nodes with properties that don't fit into either the children or string data types.
// The last three bytes of the 4-byte data field form a single 24-bit unsigned integer (i.e.,
// `uint32(0x00_ff_ff_ff & node.data)`) _N_ that is a byte offset into the **extended node data** section. The length and
// meaning of the data at that offset is defined by the node type. See the **Extended node data** section for details on
// the format of the extended data for specific node types.
//
// Encoding Arbitrary Nodes
// ------------------------
//
// The same binary format can be used to encode an arbitrary subtree of a SourceFile, not just a whole SourceFile. When
// encoding a non-SourceFile node, the format is identical with the following differences:
//
// - The content hash fields in the header (bytes 4-19) are zero.
// - The parse options field in the header (bytes 20-23) is zero.
// - The root node in the nodes section uses its actual node kind and data encoding (via getNodeData) rather than the
//   SourceFile-specific extended data format.
//
// The string data section contains only the strings referenced by nodes in the subtree, rather than the full source
// file text. The EncodeNode function provides this entrypoint.

/// Go `xxh3.Uint128`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Uint128 {
    hi: u64,
    lo: u64,
}




/// Go `Uint128{Hi, Lo}` of a 128-bit hash.
fn uint128_of(h: u128) -> Uint128 {
    Uint128 {
        hi: (h >> 64) as u64,
        lo: h as u64,
    }
}




/// The same field of either `SourceFileFields` variant.
macro_rules! source_file_field {
    ($fields:expr, $field:ident) => {
        match $fields {
            SourceFileFields::Published(info) => &info.$field,
            SourceFileFields::Parsed(file) => &file.$field,
        }
    };
}



/// Go `%v` of an `ast.Kind` (the generated stringer): "KindX", or "Kind(N)"
/// out of range.
// PORT: `crate::astdata::SyntaxKind::as_str` gives the Go names without the `Kind`
// prefix. Go `ast.Kind` is an `int16`.
pub fn go_kind_string(kind: i16) -> String {
    if kind >= 0
        && let Ok(k) = SyntaxKind::try_from(kind as u16)
    {
        return format!("Kind{}", k.as_str());
    }
    format!("Kind({kind})")
}

// Go: api/encoder/encoder.go:312 SourceFileHash

// Go: api/encoder/encoder.go:318 encodeParseOptions
/// encodeParseOptions encodes the per-file ExternalModuleIndicatorOptions as a uint32 bitmask.
fn encode_parse_options(opts: ExternalModuleIndicatorOptions) -> u32 {
    let mut bits: u32 = 0;
    if opts.jsx {
        bits |= 1;
    }
    if opts.force {
        bits |= 2;
    }
    bits
}

// Go: api/encoder/encoder.go:330 NodeIndexTable
/// NodeIndexTable maps between AST nodes and their encoder indices for O(1) node handle resolution.
// PORT: Go `sortedOnce sync.Once` and `sortedIdx []uint32` are one `OnceCell`.
pub struct NodeIndexTable {
    /// index → node (for resolution). `Node::NIL` for a NodeList slot.
    pub nodes: Vec<Node>,
    /// indices into Nodes, sorted by node ID; built lazily
    sorted_idx: OnceCell<Vec<u32>>,
}

// Go: api/encoder/encoder.go:336 nodeIndexTableKey

impl NodeIndexTable {
    // Go: api/encoder/encoder.go:342 (*NodeIndexTable).GetIndex
}

/// PORT: the counter and table that Go `BuildNodeIndexTable` closures
/// capture. They live in the visitor's `ctx` (see `ast/visitor.rs`).
struct BuildNodeIndexTableState {
    node_count: u32,
    node_table: Vec<Node>,
    source_file: Node,
}

// Go: api/encoder/encoder.go:370 BuildNodeIndexTable

// Go: api/encoder/encoder.go:416 GetNodeIndexTable

// Go: api/encoder/encoder.go:422 EncodeSourceFile



// Go: api/encoder/encoder.go SetSourceFileLease (ts#64434)

// Go: api/encoder/encoder.go:442 EncodeNode

/// PORT: the local variables that Go `encodeTree` closures capture. They
/// live in the visitor's `ctx` (see `ast/visitor.rs`), so the callbacks
/// reach them through the visitor they get.
struct EncodeTreeState {
    parent_index: u32,
    node_count: u32,
    prev_index: u32,
    extended_data: Vec<u8>,
    structured_data: Vec<u8>,
    strs: StringTable,
    position_map: Cow<'static, PositionMap>,
    nodes: Vec<u8>,
    node_table: Vec<Node>,
    node_index_map: Option<FxHashMap<Node, u32>>,
    source_file: Node,
}

impl EncodeTreeState {
}

// Go: api/encoder/encoder.go:446 encodeTree
// PORT: `given` is the parsed-file record of `source_file` when the caller
// holds it (`encode_parsed_source_file`, where `root_node` is `source_file`).
// It gives the Go `SourceFile` fields `Hash`, `ParseOptions()`, `NodeCount`,
// `TextCount` and the parser fields (`source_file_fields`).


#[cfg(test)]
thread_local! {
    /// Tests: `encode_tree` encodes every node with the visitor, as Go does.
    static VISITOR_ONLY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// True when `encode_tree` encodes every node with the visitor (tests).
#[inline]
fn visitor_only() -> bool {
    #[cfg(test)]
    return VISITOR_ONLY.with(std::cell::Cell::get);
    #[cfg(not(test))]
    false
}


/// True when `encode_visit` encodes the children of a node of kind `kind`
/// of the store file from its node data: Go `VisitEachChild` visits the
/// fields of `ForEachChild` (`for_each_child_field`) in the same order.
/// JSDoc nodes and a SyntaxList take the visitor.
#[inline]
fn walks_node_data(kind: SyntaxKind) -> bool {
    !is_js_doc_kind(kind) && kind != SyntaxKind::SyntaxList
}


// Go: api/encoder/encoder.go:540 (the visitor.Visit func of encodeTree)




// Go: api/encoder/encoder.go:503 (the VisitNodes hook of encodeTree)



// Go: api/encoder/encoder.go:655 appendUint32s



// Go: api/encoder/encoder.go:662 getNodeData
// PORT: `given` is the record that `encode_tree` was given for a SourceFile
// `node` (see `source_file_fields`), else None.

// Go: api/encoder/encoder.go:676 noStructuredData
const NO_STRUCTURED_DATA: u32 = 0xFFFFFFFF;

// Go: api/encoder/encoder.go:678 recordExtendedData_SourceFile
// PORT: `given` as in `get_node_data`.

// Go: api/encoder/encoder.go:714 recordExtendedData_TemplateHead

// Go: api/encoder/encoder.go:721 recordExtendedData_TemplateMiddle

// Go: api/encoder/encoder.go:728 recordExtendedData_TemplateTail

// Go: api/encoder/encoder.go:735 boolToByte
pub fn bool_to_byte(b: bool) -> u8 {
    if b {
        return 1;
    }
    0
}

// Go: api/encoder/encoder.go:743 hasModifiers

// Go: api/encoder/encoder.go:750 encodeFileReferences

// Go: api/encoder/encoder.go:771 encodeNodeIndexArray
/// encodeNodeIndexArray encodes a slice of LiteralLikeNodes as a msgpack array of
/// uint node indices. Returns the byte offset into the buffer, or noStructuredData
/// if the slice is empty.
// PORT: a nil Go map is `None`; Go reads 0 from it.
fn encode_node_index_array(
    nodes: &[Node],
    index_map: Option<&FxHashMap<Node, u32>>,
    buf: &mut Vec<u8>,
) -> u32 {
    if nodes.is_empty() {
        return NO_STRUCTURED_DATA;
    }
    let offset = buf.len() as u32;
    msgpack_write_array_header(buf, nodes.len());
    for &node in nodes {
        msgpack_write_uint(
            buf,
            index_map
                .and_then(|map| map.get(&node).copied())
                .unwrap_or(0),
        );
    }
    offset
}

// Go: api/encoder/encoder.go:786 encodeModuleAugmentations
/// encodeModuleAugmentations encodes a slice of ModuleName nodes as a msgpack array
/// of uint node indices. Returns the byte offset into the buffer, or noStructuredData
/// if the slice is empty.
// PORT: a nil Go map is `None`; Go reads 0 from it.
fn encode_module_augmentations(
    nodes: &[Node],
    index_map: Option<&FxHashMap<Node, u32>>,
    buf: &mut Vec<u8>,
) -> u32 {
    if nodes.is_empty() {
        return NO_STRUCTURED_DATA;
    }
    let offset = buf.len() as u32;
    msgpack_write_array_header(buf, nodes.len());
    for &node in nodes {
        msgpack_write_uint(
            buf,
            index_map
                .and_then(|map| map.get(&node).copied())
                .unwrap_or(0),
        );
    }
    offset
}

// Go: api/encoder/encoder.go:800 encodeStringArray
/// encodeStringArray encodes a slice of strings as a msgpack array of strings.
/// Returns the byte offset into the buffer, or noStructuredData if the slice is empty.
fn encode_string_array(strs: &[String], buf: &mut Vec<u8>) -> u32 {
    if strs.is_empty() {
        return NO_STRUCTURED_DATA;
    }
    let offset = buf.len() as u32;
    msgpack_write_array_header(buf, strs.len());
    for s in strs {
        msgpack_write_string(buf, s);
    }
    offset
}

// Go: api/encoder/encoder.go:812 encodeSpanMap

// Go: api/encoder/encoder.go:841 encodeDiagnosticDirectives

// Minimal msgpack writers for the structured data section.

// Go: api/encoder/encoder.go:865 msgpackWriteArrayHeader
fn msgpack_write_array_header(buf: &mut Vec<u8>, length: usize) {
    if length <= 0x0f {
        buf.push(0x90 | length as u8);
        return;
    }
    if length <= 0xffff {
        buf.extend_from_slice(&[0xdc, (length >> 8) as u8, length as u8]);
        return;
    }
    buf.extend_from_slice(&[
        0xdd,
        (length >> 24) as u8,
        (length >> 16) as u8,
        (length >> 8) as u8,
        length as u8,
    ]);
}

// Go: api/encoder/encoder.go:875 msgpackWriteUint
fn msgpack_write_uint(buf: &mut Vec<u8>, value: u32) {
    if value <= 0x7f {
        buf.push(value as u8);
        return;
    }
    if value <= 0xff {
        buf.extend_from_slice(&[0xcc, value as u8]);
        return;
    }
    if value <= 0xffff {
        buf.extend_from_slice(&[0xcd, (value >> 8) as u8, value as u8]);
        return;
    }
    buf.extend_from_slice(&[
        0xce,
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    ]);
}

// Go: api/encoder/encoder.go:888 msgpackWriteString
fn msgpack_write_string(buf: &mut Vec<u8>, s: &str) {
    let n = s.len();
    if n <= 0x1f {
        buf.push(0xa0 | n as u8);
    } else if n <= 0xff {
        buf.extend_from_slice(&[0xd9, n as u8]);
    } else if n <= 0xffff {
        buf.extend_from_slice(&[0xda, (n >> 8) as u8, n as u8]);
    } else {
        buf.extend_from_slice(&[
            0xdb,
            (n >> 24) as u8,
            (n >> 16) as u8,
            (n >> 8) as u8,
            n as u8,
        ]);
    }
    buf.extend_from_slice(s.as_bytes());
}

// Go: api/encoder/encoder.go:902 msgpackWriteBool
fn msgpack_write_bool(buf: &mut Vec<u8>, value: bool) {
    if value {
        buf.push(0xc3);
        return;
    }
    buf.push(0xc2);
}

// Hand-written commonData encoding functions for nodes whose non-bool data
// members cannot be automatically encoded by the generator. Each function
// packs relevant fields into the 6-bit commonData area (bits 24-29) of the
// 32-bit node data word.

// Go: api/encoder/encoder.go:914 getNodeCommonData_SyntheticExpression
pub fn get_node_common_data_synthetic_expression(_node: Node) -> u32 {
    // SyntheticExpression is an internal compiler node that is never part of a parsed AST.
    // It should never be encoded.
    panic!("SyntheticExpression should never be encoded")
}

// Hand-written extended data encoding functions for literal nodes that were
// previously string-type but whose TokenFlags/TemplateFlags cannot fit in 6 bits.

// Go: api/encoder/encoder.go:923 recordExtendedData_StringLiteral

// Go: api/encoder/encoder.go:929 recordExtendedData_NumericLiteral

// Go: api/encoder/encoder.go:935 recordExtendedData_BigIntLiteral

// Go: api/encoder/encoder.go:941 recordExtendedData_RegularExpressionLiteral

// Go: api/encoder/encoder.go:947 recordExtendedData_NoSubstitutionTemplateLiteral

#[cfg(test)]
mod store_data_tests {
    use super::*;
    use crate::ast::{FileText, FileVersion};
    use crate::frontend::parser::{SourceFileParseOptions, parse_source_file};

    /// Texts with every syntax kind that a parse makes outside JSDoc.
    const SAMPLES: &[(&str, &str)] = &[
        (
            "a.ts",
            r#"import def, { a as b, type c } from "./m";
import * as ns from "./n";
import x = require("x");
import type { T } from "./t" with { type: "json" };
export { b as default2, c };
export * from "./star";
export * as nsx from "./nsx";
export default class<T extends object = {}> extends Base<T> implements I, J {
    static #p?: number = 1;
    declare readonly q!: string;
    @dec() @dec2 method<U>(this: X, a?: U, ...rest: U[]): asserts a is U { super.m(); }
    get g(): number { return this.#p ?? 0; }
    set g(v) {}
    constructor(private readonly z: number, public w?: string) { super(); }
    static { label: for (;;) { break label; } }
    [key: string]: any;
    m2?(): void;
    *gen() { yield* other(); yield 1; }
    async am() { await p; for await (const v of it) {} }
    "quoted"() {}
    [computed]: number;
    123: number;
}
abstract class A { abstract m(): void; protected abstract get p(): number; }
function f(overload: string): void;
function f(overload: number): void;
function f(overload: any) {}
declare function df(): void;
declare module "mod" { export const x: number; }
declare global { interface Window { w: number } }
namespace N.M { export namespace O {} }
module Old {}
enum E { A = 1, B = A << 2, "C" }
const enum CE { X }
type U<T> = T extends (infer V extends string)[] ? V : never;
type M = { readonly [K in keyof T as `get${K & string}`]-?: T[K] };
type F = new (...a: any[]) => unknown;
type F2 = abstract new () => void;
type Tup = [a: string, b?: number, ...c: boolean[]];
type L = "lit" | 1 | -1 | true | null | undefined | 1n | `t${string}x${number}`;
type Q = typeof import("./q", { with: { "resolution-mode": "import" } }).Z<number>;
type Idx = T["k"][number];
type Pred = (x: unknown) => x is string;
type Op = keyof T | unique symbol | readonly string[];
type Paren = (string | number)[];
type Opt = { a?: number; b(): void; new (): X; (): Y; get c(): number; set c(v: number) };
type Q2 = typeof x.y<string>;
interface I extends J, K<number> { [i: number]: string; m?<T>(): void }
let v1: any = <any>x, v2 = x as const, v3 = x satisfies T, v4 = x!;
var { a1, b1: [c1, , ...d1] = [], ...e1 } = obj, [f1 = 1] = arr;
const o = { a, b: 1, [c]: 2, ...d, m() {}, get g() { return 1; }, set s(v) {}, async *ag() {}, "s": 1, 2: 3 };
const arrow = async <T,>(a: T): Promise<T> => a, arrow2 = x => ({ x });
const fe = function* named() {}, ce = class Named {};
const t = tag<string>`a${b}c${d}e`, nst = `plain`, re = /ab+c/gi, big = 123n, num = 0x1F, str = 'q\'s';
if (a) b; else if (c) { d; } else e;
do x++; while (--y);
while (true) continue;
for (let i = 0, j; i < 10; i++) {}
for (const k in o) {}
for (const [k, v] of m) {}
switch (s) { case 1: case 2: break; default: throw new Error(`e`); }
try { a(); } catch { } finally { }
try { a(); } catch (e: unknown) { }
with (o) { }
debugger;
;
x = a ? b : c, y ||= z, w ??= v, u **= 2;
delete o.p; void 0; typeof x; ++x; x--; -x; ~x; !x; +x;
new Foo<number>(1)?.bar?.[0]?.(2);
new.target; import.meta.url; import("dyn"); super.x;
a?.b!.c;
label2: { }
using res = getRes();
export = something;
"#,
        ),
        (
            "b.tsx",
            r#"const el = <div className="a" {...props} key={1} data-x='y' ns:attr="z" disabled>
    text {expr} {/* comment */} {...spread}
    <Child<number> a={<b />} />
    <></>
    <ns:tag />
    <a.b.c>more</a.b.c>
</div>;
const g = <T,>(x: T) => x;
"#,
        ),
        (
            "c.js",
            r#"/** @type {number} */
var n = 1;
/**
 * @param {string} a desc
 * @param {{ x: number }} [b]
 * @returns {Promise<void>}
 * @template T
 * @typedef {Object} Td
 * @property {string} p
 * @callback Cb
 * @see {@link Foo} and {@linkcode Bar} {@linkplain Baz}
 * @deprecated
 */
function jsf(a, b) { return a; }
/** @enum {string} */
const En = { A: "a" };
class JC { /** @private */ p = 1; #q; static s; }
module.exports = { jsf };
exports.x = 1;
"#,
        ),
        (
            "d.d.ts",
            "declare const x: number;\nexport declare function f(): void;\nexport as namespace NS;\n",
        ),
        (
            "e.ts",
            "let s = '\\ud800'; let id\\u0061 = 1; let \u{e9} = 'é'; // non-ASCII\nconst bad = `\\x`;\n",
        ),
        ("f.ts", "class C { m( { } \nlet = ; }} ) => ;\nfunction (\n"),
        // followups32: fields that the texts above leave out. A JS
        // function's `@type` gives its `FullSignature`, which has no mask
        // bit (parser/reparser.go:396), and an assignment declaration's
        // `@type` gives its BinaryExpression a `Type` (:377).
        (
            "g.js",
            r#"/** @type {(a: string, b?: number) => void} */
function typed(a, b) {}
/** @type {number} */
exports.y = 1;
/** @type {string} */
module.exports.z = "z";
"#,
        ),
        // A shorthand property with an initializer in a destructuring
        // assignment, a definite assignment `!`, optional tuple members and
        // two MissingDeclarations: decorators with no declaration
        // (parser/parser.go:1188) and a decorated expression (:5790).
        (
            "h.ts",
            "({ a = 1, b: [c] = [] } = o);\nlet x!: number;\ntype OptTup = [string?, number?];\n@dec;\nconst md = @dec 1;\n",
        ),
    ];

    /// Encodes `text` as `file_name` with the node data walk and with the
    /// visitor only, and asserts equal bytes. `freeable`: a freeable file
    /// version with owned nodes (an API source file lease), else a static
    /// publish. Both are published and bound, as the parse cache of the API
    /// does (`project::acquire_bound`).
    fn assert_same_encoding(file_name: &str, text: &str, freeable: bool) {
        let options = SourceFileParseOptions {
            file_name: file_name.to_string(),
            path: tspath::Path(file_name.to_string()),
            ..Default::default()
        };
        let script_kind = crate::frontend::core_ext::ensure_script_kind_from_file_name(file_name);
        let file = {
            let _scope = freeable.then(crate::ast::store::enter_owned_parse);
            parse_source_file(
                &options,
                FileText::new(text.to_string(), freeable),
                script_kind,
            )
        };
        if freeable {
            assert!(file.version.set(FileVersion::new(file.store)).is_ok());
        }
        let file = Rc::new(file);
        crate::program::note_parsed_source_file(&file);
        crate::program::publish_parsed_files("/");
        crate::program::bind_file_outside_program(file.root);
        match FileNodes::of(file.root.file_index()) {
            Some(FileNodes::Pinned(_)) => assert!(freeable, "{file_name}: a freeable version"),
            Some(FileNodes::Static(_)) => assert!(!freeable, "{file_name}: a static publish"),
            None => panic!("{file_name}: the file is not published"),
        }
        let (walk, _) = encode_parsed_source_file(&file).expect("encode");
        VISITOR_ONLY.with(|v| v.set(true));
        let (visitor, _) = encode_parsed_source_file(&file).expect("encode");
        VISITOR_ONLY.with(|v| v.set(false));
        assert!(
            walk == visitor,
            "{file_name}: the node data walk and the visitor differ"
        );
        drop(file);
        // The pins of this thread hold a freeable version until here.
        crate::ast::release_file_version_pins();
    }

    #[test]
    fn the_node_data_walk_encodes_as_the_visitor() {
        for (i, (name, text)) in SAMPLES.iter().enumerate() {
            assert_same_encoding(&format!("/apiperf2/static/{i}/{name}"), text, false);
            assert_same_encoding(&format!("/apiperf2/freeable/{i}/{name}"), text, true);
        }
        // Not run by the test suite: set GOPORT_ENCODE_CORPUS to directories
        // (`:` between them) to compare every .ts, .tsx, .js, .jsx, .mts and
        // .cts file under them, outside node_modules.
        let Ok(dirs) = std::env::var("GOPORT_ENCODE_CORPUS") else {
            return;
        };
        let mut stack: Vec<std::path::PathBuf> =
            dirs.split(':').map(std::path::PathBuf::from).collect();
        let mut count = 0;
        while let Some(dir) = stack.pop() {
            let mut entries: Vec<_> = std::fs::read_dir(&dir)
                .expect("read the corpus directory")
                .map(|e| e.expect("a corpus entry").path())
                .collect();
            entries.sort();
            for path in entries {
                if path.is_dir() {
                    if !path.ends_with("node_modules") {
                        stack.push(path);
                    }
                    continue;
                }
                let name = path.to_string_lossy().to_string();
                if ![".ts", ".tsx", ".js", ".jsx", ".mts", ".cts"]
                    .iter()
                    .any(|e| name.ends_with(e))
                {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                count += 1;
                // A static publish is never freed: only small files.
                if text.len() <= 16 * 1024 {
                    assert_same_encoding(&format!("/apiperf2/c{count}/static{name}"), &text, false);
                }
                assert_same_encoding(&format!("/apiperf2/c{count}/freeable{name}"), &text, true);
            }
        }
        eprintln!("compared {count} corpus files");
    }
}
