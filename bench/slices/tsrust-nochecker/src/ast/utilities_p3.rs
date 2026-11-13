//! Go `internal/ast/utilities.go` lines 1806-2728.

use crate::prelude::*;

// Go: ast/utilities.go:1864 GetSuperContainer

// Go: ast/utilities.go:1892 GetImmediatelyInvokedFunctionExpression

// Go: ast/utilities.go:1907 IsEnumConst

// Go: ast/utilities.go:1911 ExpressionIsAlias

// Go: ast/utilities.go:1915 IsInstanceOfExpression

// Go: ast/utilities.go:1919 IsAnyImportOrReExport

// Go: ast/utilities.go:1923 IsImportNode

// Checks if the node is a genuine import declation. In particular the re-parsed KindJSImportDeclaration
// is explicitly excluded because the callers of this function are typically not prepared to handle it properly.
// For more permissive check, use IsImportNode.
// Go: ast/utilities.go:1930 IsAnyImportSyntax

// Go: ast/utilities.go:1934 IsJsonSourceFile

// Go: ast/utilities.go:1938 IsInJsonFile

// Go: ast/utilities.go:1942 GetExternalModuleName

// Go: ast/utilities.go:1964 HasImportAttributes (ts#63931)

// Go: ast/utilities.go:1972 GetImportAttributes

// Go: ast/utilities.go:1984 getImportTypeNodeLiteral

// Go: ast/utilities.go:1997 IsExpressionNode

// Go: ast/utilities.go:2032 IsInExpressionContext

// Go: ast/utilities.go:2058 IsPartOfTypeNode

// Go: ast/utilities.go:2089 isPartOfTypeNodeInParent

// Go: ast/utilities.go:2123 isPartOfTypeExpressionWithTypeArguments

// Go: ast/utilities.go:2130 IsJSDocLinkLike

// Go: ast/utilities.go:2134 IsJSDocTag

// Go: ast/utilities.go:2138 IsSuperCall

// Go: ast/utilities.go:2142 IsImportCall

// Go: ast/utilities.go:2150 IsComputedNonLiteralName

// Go: ast/utilities.go:2154 IsQuestionToken

// PORT: Go `getTextOfNode func(*Node) string` may be nil, so it is an Option.
// Go: ast/utilities.go:2158 EntityNameToString

// Go: ast/utilities.go:2177 GetTextOfPropertyName

// Go: ast/utilities.go:2182 TryGetTextOfPropertyName

// Go: ast/utilities.go:2197 IsJSDocNode

// Go: ast/utilities.go:2201 IsNonWhitespaceToken

// Go: ast/utilities.go:2205 IsWhitespaceOnlyJsxText

// Go: ast/utilities.go:2209 GetNewTargetContainer

// Go: ast/utilities.go:2220 GetEnclosingBlockScopeContainer

// Go: ast/utilities.go:2226 IsBlockScope

// Go type SemanticMeaning (ast/utilities.go:2172) lives in crate::flags.

// Go: ast/utilities.go:2250 GetMeaningFromDeclaration

// Go: ast/utilities.go:2308 IsPropertyAccessOrQualifiedName

// Go: ast/utilities.go:2312 IsLabelName

// Go: ast/utilities.go:2316 IsLabelOfLabeledStatement

// Go: ast/utilities.go:2326 IsJumpStatementTarget

// Go: ast/utilities.go:2336 IsBreakOrContinueStatement

// GetModuleInstanceState is used during binding as well as in transformations and tests, and therefore may be invoked
// with a node that does not yet have its `Parent` pointer set. In this case, an `ancestors` represents a stack of
// virtual `Parent` pointers that can be used to walk up the tree. Since `getModuleInstanceStateForAliasTarget` may
// potentially walk up out of the provided `Node`, merely setting the parent pointers for a given `ModuleDeclaration`
// prior to invoking `GetModuleInstanceState` is not sufficient. It is, however, necessary that the `Parent` pointers
// for all ancestors of the `Node` provided to `GetModuleInstanceState` have been set.

// Push a virtual parent pointer onto `ancestors` and return it.
// PORT: Go `append` on a slice value; we return a new Vec so callers never share a backing array.
// Go: ast/utilities.go:2348 pushAncestor
pub fn push_ancestor(ancestors: &[Node], parent: Node) -> Vec<Node> {
    let mut result = ancestors.to_vec();
    result.push(parent);
    result
}

// If a virtual `Parent` exists on the stack, returns the previous stack entry and the virtual `Parent`.
// Otherwise, we return `nil` and the value of `node.Parent`.
// Go: ast/utilities.go:2354 popAncestor

// Go type ModuleInstanceState (ast/utilities.go:2294) lives in crate::flags.

// Go: ast/utilities.go:2371 GetModuleInstanceState

// PORT: Go `getModuleInstanceState` has the same snake name as the exported
// `GetModuleInstanceState`. The exported one keeps the plain name because
// other packages call it; this private one gets the `_unexported` suffix.
// `visited` is keyed by the Node handle instead of Go `NodeId` (same identity).
// Go: ast/utilities.go:2375 getModuleInstanceState

// Go: ast/utilities.go:2384 getModuleInstanceStateCached

// Go: ast/utilities.go:2401 getModuleInstanceStateWorker

// Go: ast/utilities.go:2455 getModuleInstanceStateForAliasTarget

// Go: ast/utilities.go:2492 IsInstantiatedModule

// Go: ast/utilities.go:2498 NodeHasName

// Go: ast/utilities.go:2510 IsInternalModuleImportEqualsDeclaration

// Go: ast/utilities.go:2514 IsConstAssertion


// Go: ast/utilities.go:2522 IsConstTypeReference
// PERF: U1 (a). Compares name ids (`Node::text_is`), with no text load.

// Go: ast/utilities.go:2526 IsGlobalSourceFile

// Go: ast/utilities.go:2530 IsParameterLike

// Go: ast/utilities.go:2538 GetDeclarationOfKind

// Go: ast/utilities.go:2547 FindConstructorDeclaration

// Go: ast/utilities.go:2556 GetFirstIdentifier

// Go: ast/utilities.go:2568 GetNamespaceDeclarationNode

// Go: ast/utilities.go:2588 ModuleExportNameIsDefault

// Go: ast/utilities.go:2592 IsDefaultImport

// PORT: Go `tspath.FileExtensionIsOneOf`; tspath has no port in this crate, so
// this private copy follows `tspath.FileExtensionIs` exactly.
fn file_extension_is_one_of_p3(path: &str, extensions: &[&str]) -> bool {
    extensions
        .iter()
        .any(|ext| path.len() > ext.len() && path.ends_with(ext))
}

// Go: ast/utilities.go:2601 GetImpliedNodeFormatForFile
pub fn get_implied_node_format_for_file(path: &str, package_json_type: &str) -> ModuleKind {
    // Go core.ResolutionModeNone / ESM / CommonJS are ModuleKind None / ESNext / CommonJS.
    let mut implied_node_format = ModuleKind::NONE;
    if file_extension_is_one_of_p3(path, &[".d.mts", ".mts", ".mjs"]) {
        implied_node_format = ModuleKind::ES_NEXT;
    } else if file_extension_is_one_of_p3(path, &[".d.cts", ".cts", ".cjs"]) {
        implied_node_format = ModuleKind::COMMON_JS;
    } else if file_extension_is_one_of_p3(path, &[".d.ts", ".ts", ".tsx", ".js", ".jsx"]) {
        implied_node_format = if package_json_type == "module" {
            ModuleKind::ES_NEXT
        } else {
            ModuleKind::COMMON_JS
        };
    }

    implied_node_format
}

// Go: ast/utilities.go:2614 GetEmitModuleFormatOfFileWorker

// Go: ast/utilities.go:2622 GetImpliedNodeFormatForEmitWorker
pub fn get_implied_node_format_for_emit_worker(
    file_name: &str,
    emit_module_kind: ModuleKind,
    source_file_meta_data: &SourceFileMetaData,
) -> ModuleKind {
    if ModuleKind::NODE16 <= emit_module_kind && emit_module_kind <= ModuleKind::NODE_NEXT {
        return source_file_meta_data.implied_node_format;
    }
    if source_file_meta_data.implied_node_format == ModuleKind::COMMON_JS
        && (source_file_meta_data.package_json_type == "commonjs"
            || file_extension_is_one_of_p3(file_name, &[".cjs", ".cts"]))
    {
        return ModuleKind::COMMON_JS;
    }
    if source_file_meta_data.implied_node_format == ModuleKind::ES_NEXT
        && (source_file_meta_data.package_json_type == "module"
            || file_extension_is_one_of_p3(file_name, &[".mjs", ".mts"]))
    {
        return ModuleKind::ES_NEXT;
    }
    ModuleKind::NONE
}

// Go: ast/utilities.go:2639 GetDeclarationContainer

// Indicates that a symbol is an alias that does not merge with a local declaration.
// OR Is a JSContainer which may merge an alias with a local declaration
// Go: ast/utilities.go:2657 IsNonLocalAlias

// An alias symbol is created by one of the following declarations:
//
//	import <symbol> = ...
//	const <symbol> = ... (JS only)
//	const { <symbol>, ... } = ... (JS only)
//	import <symbol> from ...
//	import * as <symbol> from ...
//	import { x as <symbol> } from ...
//	export { x as <symbol> } from ...
//	export * as ns <symbol> from ...
//	export = <EntityNameExpression>
//	export default <EntityNameExpression>
//	module.exports = <EntityNameExpression> (JS only)
//	module.exports.<symbol> = <EntityNameExpression> (JS only)
//	exports.<symbol> = <EntityNameExpression> (JS only)
// Go: ast/utilities.go:2680 IsAliasSymbolDeclaration

// Go: ast/utilities.go:2700 IsParseTreeNode

// Returns a token if position is in [start-of-leading-trivia, end), includes JSDoc only if requested
// Go: ast/utilities.go:2705 GetNodeAtPosition

// Go: ast/utilities.go:2733 nodeContainsPosition

/// `get_node_at_position` for a series of positions in one tree that does
/// not change between the calls. It gives the node that
/// `get_node_at_position` gives for each position. It is fast when the
/// positions never go down; a smaller position descends from the root.
// PERF: loadcrit1 step 0. `for_each_dynamic_import_or_require_call` looks up
// each `import`/`require` word. Go descends from the root for each word and
// checks the children of each level in order (ast/utilities.go:2705). A
// `.d.ts` file with a wide type literal (@redis/client
// `commands/index.d.ts`: 743 members, 10,783 words) makes that
// O(words x members). In the port each check also reads a store record that
// the parse has not published yet (`loc_slow`, `kind_slow`), so this one
// parse held the umami and directus loads for about 80 ms.
//
// The cursor also descends from the root for each position, and gives the
// same first child at each level as Go's scan. It keeps the levels of the
// last descent: each node with its children in `for_each_child` order and
// `skip`, the count of leading children that contain no position from the
// last one on. A later position only grows, so the scan of a level starts
// at `skip`, and the children it passes cannot contain the position. This
// holds for any tree shape (overlapping or unsorted children, JSDoc), so
// the JS files use the cursor too.
#[derive(Default)]
struct NodeAtPositionCursor {
    /// The levels of the last descent, root first. A level past the
    /// current depth is the state of a node that the descent left; it is
    /// used again only for the same node.
    levels: Vec<CursorLevel>,
    last_position: i32,
}

struct CursorLevel {
    node: Node,
    children: Vec<Node>,
    /// No child before this index contains a position from
    /// `NodeAtPositionCursor::last_position` on.
    skip: usize,
}

impl CursorLevel {



}

impl NodeAtPositionCursor {
    // Go: ast/utilities.go:2705 GetNodeAtPosition
}

// Go: ast/utilities.go:2737 findImportOrRequire

// Go: ast/utilities.go:2764 ForEachDynamicImportOrRequireCall

#[cfg(test)]
mod node_at_position_cursor_tests {
    use super::*;
    use crate::frontend::parser::{SourceFileParseOptions, parse_source_file};
    use crate::frontend::tspath::Path;

    fn parse(name: &str, text: &'static str, kind: ScriptKind) -> Node {
        parse_source_file(
            &SourceFileParseOptions {
                file_name: name.to_string(),
                path: Path(name.to_string()),
                ..Default::default()
            },
            text,
            kind,
        )
        .root
    }

    /// The cursor gives the node of `get_node_at_position` at every
    /// position in order, and at every `import`/`require` word (the
    /// positions of `for_each_dynamic_import_or_require_call`, also in
    /// comments, strings and names), with and without JSDoc.
    fn check(file: Node) {
        let text = source_file_text(file);
        let mut words = Vec::new();
        let (mut index, mut size) = find_import_or_require(&text, 0);
        while index >= 0 {
            words.push(index);
            (index, size) = find_import_or_require(&text, index + size);
        }
        assert!(!words.is_empty());
        let every = (0..=text.len() as i32).collect::<Vec<_>>();
        for include_js_doc in [false, true] {
            for positions in [&every, &words] {
                let mut cursor = NodeAtPositionCursor::default();
                for &position in positions {
                    assert_eq!(
                        cursor.node_at(file, position, include_js_doc),
                        get_node_at_position(file, position, include_js_doc),
                        "position {position}, JSDoc {include_js_doc}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_cursor_finds_the_node_of_a_root_descent() {
        // A wide type literal, as @redis/client `commands/index.d.ts`.
        let mut wide = String::from("// import require\ndeclare const _default: {\n");
        for i in 0..300 {
            wide += &format!(
                "    /** import {i} */ m{i}: typeof import(\"./m{i}\"); importantRequire{i}: import(\"./t{i}\").T<\"require\">;\n"
            );
        }
        wide += "    // import before the brace\n};\nexport default _default; // require\n";
        check(parse("/wide.d.ts", wide.leak(), ScriptKind::TS));

        check(parse(
            "/a.ts",
            "import x from \"x\";\nexport * from \"./require\";\n\
             const m = import.meta; const n = import.meta.url;\n\
             async function f() { await import(\"a\"); return import(`b`).then(() => import(c)); }\n\
             type T = typeof import(\"t\") extends import(\"u\").U ? 1 : 2;\n\
             class C { @dec(import(\"d\")) static [require(\"k\")]: import(\"v\").V; }\n\
             declare module \"require\" { export type R = import(\"r\").R; }\n\
             // import(\"in a comment\") require(\"x\")\n\
             const s = \"import('a string') require\"; /* import */",
            ScriptKind::TS,
        ));

        check(parse(
            "/a.js",
            "const a = require(\"a\");\n\
             /** @type {import(\"b\").B} */\n\
             const b = require(`b`).b;\n\
             /**\n * @typedef {import('c').C} C\n * @param {import(\"d\").D} d\n * @returns {Promise<import(\"e\")>}\n */\n\
             function f(d) { return import(\"e\"); }\n\
             module.exports = { requireIt: () => require(\"f\") };\n\
             /** @import { G } from \"g\" */\n",
            ScriptKind::JS,
        ));

        // Error recovery: missing and zero-width nodes, unclosed lists.
        check(parse(
            "/broken.ts",
            "const a = import(; type T = import(\nclass { import( }\n\
             function require(x, { import\nlet [b, require(\"c\")] = import(\"d\"\n\
             export default async function* ( require ) => import\n\
             <div>{import(\"e\")} require</div>\n\
             type U = { [K in import(\"f\")]: require",
            ScriptKind::TS,
        ));

        check(parse(
            "/a.tsx",
            "const x = <A b={import(\"b\")}>import require {require(\"c\")}</A>;\n\
             export const y = () => <></>; import(\"z\");",
            ScriptKind::TSX,
        ));
    }

    /// After the cursor has passed a position, a smaller one still gets the
    /// node of a root descent (loadcrit1 follow-up: Go has no cursor). The
    /// positions go up, down and up again, so a fallback that moves
    /// `last_position` down fails (the followups33 skeptic's mutant CURS).
    #[test]
    fn a_smaller_position_gets_the_node_of_a_root_descent() {
        let mut wide = String::from("declare const _default: {\n");
        for i in 0..20 {
            wide += &format!("    m{i}: typeof import(\"./m{i}\");\n");
        }
        wide += "};\nexport default _default;\n";
        let file = parse("/wide.d.ts", wide.leak(), ScriptKind::TS);
        let end = source_file_text(file).len() as i32;
        for include_js_doc in [false, true] {
            let mut cursor = NodeAtPositionCursor::default();
            for position in (0..=end).chain((0..=end).rev()).chain(0..=end) {
                assert_eq!(
                    cursor.node_at(file, position, include_js_doc),
                    get_node_at_position(file, position, include_js_doc),
                    "position {position}, JSDoc {include_js_doc}"
                );
            }
        }
    }
}
