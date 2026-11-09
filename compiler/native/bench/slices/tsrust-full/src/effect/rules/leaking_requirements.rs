//! Port of Effect-TS/tsgo `internal/rules/leaking_requirements.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// LeakingRequirements detects when service methods inadvertently expose implementation
// dependencies (Requirements) in their public type signatures. When every method of a
// service requires the same dependency types from callers, this is typically a sign that
// those dependencies should be resolved at Layer creation time instead.
// Supports both V3 and V4, default severity suggestion.

// Collect types to check and the node to report on



// PORT: Go also takes the checker and returns false when it is nil; the
// port always has one, so the parameter is dropped.


// parseLeakedRequirements analyzes the service shape to find requirement types that
// are shared across all effect-typed members. This is the "leaking requirements" heuristic.

// hasLeakableServiceAnnotation checks if a type's declarations contain the
// @effect-leakable-service JSDoc annotation.

// intersectStringSlices returns the elements that appear in both slices.
fn intersect_string_slices(a: &[String], b: &[String]) -> Vec<String> {
    let set: FxHashSet<&String> = b.iter().collect();
    let mut result = Vec::new();
    for s in a {
        if set.contains(s) {
            result.push(s.clone());
        }
    }
    result
}
