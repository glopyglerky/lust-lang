//! Port of Go `ls/definition.go`.
//!
//! PORT: Go `*compiler.Program` is `&compiler::NewProgram`. A Go
//! checker lease `c, done := program.GetTypeCheckerForFile(ctx, file)` is
//! `ls_program::get_type_checker_for_file`, with `done` kept alive to the end
//! of the scope (Go `defer done()`).

use crate::ls::prelude::*;

use crate::spanmap::Feature;


// Go: ls/definition.go:162 combineDefinitionResponses
// PORT: Go takes the slice; here the results are moved in. Go appends the
// same `*LocationLink` it read; here the link is moved. Go `seen` is a
// `collections.Set[lsproto.Location]`. Go returns a pointer to a possibly
// nil slice, which marshals as `[]`; here `Some` of a possibly empty `Vec`.
#[must_use]
pub fn combine_definition_responses(
    results: Vec<lsproto::DefinitionResponse>,
    links: bool,
) -> lsproto::DefinitionResponse {
    let mut locations: Vec<lsproto::Location> = Vec::new();
    let mut definition_links: Vec<lsproto::LocationLink> = Vec::new();
    let mut seen: FxHashSet<lsproto::Location> = FxHashSet::default();
    for result in results {
        if let Some(result_links) = result.definition_links {
            for link in result_links {
                let location = lsproto::Location {
                    uri: link.target_uri.clone(),
                    range: link.target_selection_range,
                };
                if seen.insert(location.clone()) {
                    definition_links.push(link);
                    locations.push(location);
                }
            }
        }
        if let Some(location) = result.location {
            if seen.insert(location.clone()) {
                definition_links.push(lsproto::LocationLink {
                    origin_selection_range: None,
                    target_uri: location.uri.clone(),
                    target_range: location.range,
                    target_selection_range: location.range,
                });
                locations.push(location);
            }
        }
        if let Some(result_locations) = result.locations {
            for location in result_locations {
                if seen.insert(location.clone()) {
                    definition_links.push(lsproto::LocationLink {
                        origin_selection_range: None,
                        target_uri: location.uri.clone(),
                        target_range: location.range,
                        target_selection_range: location.range,
                    });
                    locations.push(location);
                }
            }
        }
    }
    if links {
        return lsproto::LocationOrLocationsOrDefinitionLinksOrNull {
            definition_links: Some(definition_links),
            ..Default::default()
        };
    }
    lsproto::LocationOrLocationsOrDefinitionLinksOrNull {
        locations: Some(locations),
        ..Default::default()
    }
}

// Go: ls/definition.go:195 getDeclarationNameForKeyword

// Go: ls/definition.go:208 fileRange
// PORT: Go `file *ast.SourceFile` compares by pointer; here the file root
// `Node`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FileRange {
    pub file: Node,
    pub file_range: TextRange,
}


// Go: ls/definition.go:281 lspRangeContains
#[must_use]
pub fn lsp_range_contains(outer: lsproto::Range, inner: lsproto::Range) -> bool {
    lsproto::compare_positions(outer.start, inner.start) <= 0
        && lsproto::compare_positions(inner.end, outer.end) <= 0
}

// Go: ls/definition.go:286 createLocationsFromLinks
pub fn create_locations_from_links(links: &[lsproto::LocationLink]) -> lsproto::DefinitionResponse {
    let locations: Vec<lsproto::Location> = links
        .iter()
        .map(|link| lsproto::Location {
            uri: link.target_uri.clone(),
            range: link.target_selection_range,
        })
        .collect();
    lsproto::LocationOrLocationsOrDefinitionLinksOrNull {
        locations: Some(locations),
        ..Default::default()
    }
}


// Go: ls/definition.go:306 getDeclarationsFromLocation

// Go: ls/definition.go:380 getDeclarationsFromObjectLiteralElement
// getDeclarationsFromObjectLiteralElement returns declarations from the contextual type
// of an object literal element, if available.

// Go: ls/definition.go:410 getAncestorCallLikeExpression
// Returns a CallLikeExpression where `node` is the target being invoked.

// Go: ls/definition.go:421 tryGetSignatureDeclaration

// Go: ls/definition.go:438 isJsxConstructorLike

// Go: ls/definition.go:450 symbolMatchesSignature
// PORT: Go reads symbol fields without a checker; the symbol arena is the
// first parameter, as for ast helpers that take a symbol.

// Go: ls/definition.go:463 getSymbolForOverriddenMember

// Go: ls/definition.go:493 getTypeOfSymbolAtLocation
// PORT: a free function; `Checker::get_type_of_symbol_at_location` is the
// checker method it calls (a method and a free fn do not clash).

// Go: ls/definition.go:506 getDeclarationsFromType
// PORT: Go reads type and symbol fields without a checker; the checker is
// the first parameter.
