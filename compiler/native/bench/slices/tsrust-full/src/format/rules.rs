use crate::format::prelude::*;

use std::sync::Arc;

// Builds a Go `[]contextPredicate{...}` literal. A bare predicate function
// becomes an `Arc`; a call such as `is_option_enabled(x)` or
// `option_equals(x, v)` already returns a `ContextPredicate`.
macro_rules! preds {
    (@one $f:ident ( $($args:tt)* )) => { $f($($args)*) };
    (@one $f:ident) => { Arc::new($f) as ContextPredicate };
    ($($f:ident $(( $($args:tt)* ))?),* $(,)?) => {
        vec![$(preds!(@one $f $(( $($args)* ))?)),*]
    };
}

// Go: format/rules.go:10 getAllRules

// Go: format/rules.go:428 tokenRangeFrom
pub fn token_range_from(tokens: &[SyntaxKind]) -> TokenRange {
    TokenRange {
        is_specific: true,
        tokens: tokens.to_vec(),
    }
}

// Go: format/rules.go:435 tokenRangeFromEx
// PORT: Go appends to `prefix` in place when it has spare capacity; see the
// note in get_all_rules for the one call site where that matters.
pub fn token_range_from_ex(prefix: &[SyntaxKind], tokens: &[SyntaxKind]) -> TokenRange {
    let mut all = prefix.to_vec();
    all.extend_from_slice(tokens);
    TokenRange {
        is_specific: true,
        tokens: all,
    }
}

// Go: format/rules.go:443 tokenRangeFromRange
pub fn token_range_from_range(start: SyntaxKind, end: SyntaxKind) -> TokenRange {
    let mut tokens: Vec<SyntaxKind> = Vec::with_capacity((end as usize) - (start as usize) + 1);
    for token in (start as u16)..=(end as u16) {
        tokens.push(SyntaxKind::try_from(token).expect("token kinds are contiguous"));
    }

    token_range_from(&tokens)
}
