//! Port of Go `ls/crossproject.go`.
//!
//! PORT notes for the whole file:
//! - Go runs the per-project searches on a parallel `core.WorkGroup` and
//!   shares `results`, `defaultDefinition`, `err` and `panicsOccurred` under
//!   mutexes. The shared locals are `RefCell` fields of one
//!   `CrossProjectState` value on the dispatch thread. The Go closures
//!   `canSearchProject`, `enqueueItem` and the queued function are its
//!   methods.
//! - The queued function of an item runs in 3 phases:
//!   1. open (dispatch thread): the item's language service, Go
//!      `GetLanguageServiceForProjectWithFile`;
//!   2. search: `provideSymbolsAndEntries`, the original definition locations
//!      and `symbolAndEntriesToResp`, with no session call. Item 0 (the
//!      default project, with the caller's language service and the default
//!      definition) runs it on the dispatch thread. So does every item of a
//!      request that passes no `search` function (VS references, incoming
//!      calls). The other items of references, implementations and rename
//!      run it on the search thread of their program (`search_thread.rs`),
//!      in parallel, as Go runs them on goroutines;
//!   3. commit (dispatch thread): for each location in search order, Go
//!      `GetProjectsForFile` and `enqueueItem`; then the response, or the
//!      first error.
//!   Items get their number when they leave the queue, and commit strictly
//!   in that order. So the enqueue calls, `results` (keys, order, winning
//!   positions) and the first error are those of a serial run in Go start
//!   order, whatever order the searches end in. An item that runs on the
//!   dispatch thread opens only after every earlier item committed, so a
//!   request with no `search` function runs as before, one item at a time.
//! - Go `collections.SyncMap.Range` order is random. `results` is an
//!   `IndexMap` in insertion order. Multi-project result order can differ
//!   from Go; the oracle compares it without order.
//! - Go `iter.Seq[Resp]` passed to `combineResults` is the slice of the
//!   values the iterator yields, in the same order. Go restarts the iterator
//!   when a combiner ranges over it again; here the combiner reads the slice
//!   again.

use crate::ls::prelude::*;

use crate::frontend::compiler;
use crate::frontend::tspath;
use crate::gostd::{GoError};
use crate::ls::search_thread;
use crate::lsp::lsproto;
use crate::lsp::lsproto::{HasLocation, HasLocations, HasTextDocumentPosition, HasTextDocumentURI};
use std::collections::VecDeque;
use std::panic::AssertUnwindSafe;
use std::sync::mpsc;

// Go: ls/crossproject.go:17 Project
// PORT: Go `GetProgram()` is nil for a project without a program (a
// solution tsconfig with `files: []` and references); the port gives `None`.

// Go: ls/crossproject.go:23 projectAndTextDocumentPosition
// PORT: Go `Project` values are `Rc<dyn Project>`; Go interface equality is
// `Rc::ptr_eq`. Go `ls *LanguageService` is set only for the default
// project (the caller's language service), so it is a borrow.

// Go: ls/crossproject.go:32 response
#[derive(Clone, Debug, Default)]
pub struct Response<Resp> {
    pub complete: bool,
    pub result: Resp,
    pub for_original_location: bool,
}

// Go: ls/crossproject.go:38 CrossProjectOrchestrator
// PORT: Go `*LanguageService` results are new language services owned by
// the caller (`Option` for nil). Go `iter.Seq[Project]` is a push iterator:
// `yield_` gets each project and returns false to stop.



// Go: ls/crossproject.go:46 (*LanguageService).handleCrossProject
// PORT: Go 1.27 makes this a generic method of the default language
// service (ts#63902); here it stays a function whose first parameter is
// `default_ls`.
// PORT: Go `params Req` is a pointer type: `&Req`. Go `orchestrator` can be
// nil: `Option<&dyn CrossProjectOrchestrator>`. Go
// `combineResults func(iter.Seq[Resp]) Resp` takes the yielded values as a
// slice (see the file header). `search` starts phase 2 of an item on a
// search thread (`search_thread::start_search::<K>`); with `None`, every item
// runs on the dispatch thread.

// Go: ls/crossproject.go:96 steps 3 to 5 of the queued function
// Phase 2 of an item (see the file header): the search, the original
// definition locations of each entry (collected into `locations` for the
// commit, where Go makes the session calls in its callback) and the
// response. `on_entry` runs first for each entry (the default definition of
// item 0). Returns None where Go returns early because the request was
// canceled.

// Go: ls/crossproject.go:96 steps 3 to 5 of the queued function
// PORT: `search_item` for an item that can carry Go `item.symbolData`
// (`symbol_data`; only item 0, on the dispatch thread).


/// Go `%v` of a recovered panic value.
pub fn panic_payload_text(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(p) = payload.downcast_ref::<crate::core::GoPanic>() {
        p.message.clone()
    } else if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        format!("{payload:?}")
    }
}






#[derive(Clone, Copy)]
enum Phase {
    Inline,
    Running,
    Done,
}



// Go: ls/crossproject.go:298 combineLocationArray
// PORT: Go `locations *[]T` is read only: `&[T]`.
pub fn combine_location_array<T: HasLocation + Clone>(
    mut combined: Vec<T>,
    locations: &[T],
    seen: &mut FxHashSet<lsproto::Location>,
) -> Vec<T> {
    for loc in locations {
        if seen.insert(loc.get_location()) {
            combined.push(loc.clone());
        }
    }
    combined
}

// Go: ls/crossproject.go:311 combineResponseLocations
// PORT: Go returns a non-nil `*[]lsproto.Location`: always `Some`.
pub fn combine_response_locations<T: HasLocations>(
    results: &[T],
) -> Option<Vec<lsproto::Location>> {
    let mut combined: Vec<lsproto::Location> = Vec::new();
    let mut seen_locations: FxHashSet<lsproto::Location> = FxHashSet::default();
    for resp in results {
        if let Some(locations) = resp.get_locations() {
            combined = combine_location_array(combined, locations, &mut seen_locations);
        }
    }
    Some(combined)
}

// Go: ls/crossproject.go:322 combineReferences
pub fn combine_references(results: &[lsproto::ReferencesResponse]) -> lsproto::ReferencesResponse {
    lsproto::LocationsOrNull {
        locations: combine_response_locations(results),
    }
}

// Go: ls/crossproject.go:326 combineVSReferences
pub fn combine_vs_references(
    results: &[lsproto::VSReferencesResponse],
) -> lsproto::VSReferencesResponse {
    let mut combined: Vec<lsproto::VSReferenceItem> = Vec::new();
    // Re-number IDs across projects to maintain unique IDs and correct definition references
    let mut next_id: i32 = 0;
    for resp in results {
        let Some(vs_reference_items) = &resp.vs_reference_items else {
            continue;
        };
        // Map old IDs to new IDs for this batch
        let mut id_map: FxHashMap<i32, i32> = FxHashMap::default();
        for item in vs_reference_items {
            let old_id = item.vs_id;
            let new_id = next_id;
            id_map.insert(old_id, new_id);
            next_id += 1;

            let mut new_item = item.clone();
            new_item.vs_id = new_id;
            if let Some(vs_definition_id) = item.vs_definition_id {
                let new_def_id = id_map.get(&vs_definition_id).copied().unwrap_or_default();
                new_item.vs_definition_id = Some(new_def_id);
            }
            combined.push(new_item);
        }
    }
    lsproto::VSReferenceItemsOrNull {
        vs_reference_items: Some(combined),
    }
}

// Go: ls/crossproject.go:354 combineImplementations
pub fn combine_implementations(
    results: &[lsproto::ImplementationResponse],
) -> lsproto::ImplementationResponse {
    let mut combined: Vec<lsproto::LocationLink> = Vec::new();
    let mut seen_locations: FxHashSet<lsproto::Location> = FxHashSet::default();
    for resp in results {
        if let Some(definition_links) = &resp.definition_links {
            combined = combine_location_array(combined, definition_links, &mut seen_locations);
        } else if resp.locations.is_some() {
            return lsproto::LocationOrLocationsOrDefinitionLinksOrNull {
                locations: combine_response_locations(results),
                ..Default::default()
            };
        }
    }
    lsproto::LocationOrLocationsOrDefinitionLinksOrNull {
        definition_links: Some(combined),
        ..Default::default()
    }
}

// Go: ls/crossproject.go:367 combineRenameResponse
// PORT: Go `combined` is a Go map, which the response marshals in random
// order. It is an `IndexMap` in first-insert order. Go ranges over each
// response's `Changes` map in random order; this uses its insertion order.

// Go: ls/crossproject.go:423 combineIncomingCalls
pub fn combine_incoming_calls(
    results: &[lsproto::CallHierarchyIncomingCallsResponse],
) -> lsproto::CallHierarchyIncomingCallsResponse {
    let mut combined: Vec<lsproto::CallHierarchyIncomingCall> = Vec::new();
    let mut seen_calls: FxHashSet<lsproto::Location> = FxHashSet::default();
    for resp in results {
        if let Some(call_hierarchy_incoming_calls) = &resp.call_hierarchy_incoming_calls {
            for call in call_hierarchy_incoming_calls {
                if seen_calls.insert(
                    call.from
                        .as_ref()
                        .unwrap_or_else(|| crate::core::go_nil_dereference())
                        .get_location(),
                ) {
                    combined.push(call.clone());
                }
            }
        }
    }
    lsproto::CallHierarchyIncomingCallsOrNull {
        call_hierarchy_incoming_calls: Some(combined),
    }
}
