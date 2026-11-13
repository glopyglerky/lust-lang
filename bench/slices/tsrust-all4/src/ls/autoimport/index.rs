use crate::ls::autoimport::prelude::*;

use crate::gostd::unicode;

// Port of Go `ls/autoimport/index.go`.
//
// PORT: Go `rune` keys are `char`. Names are Go strings in the port form
// (see `scanner_util::GO_STRING_MARKER`). Go reads each byte that is not
// valid UTF-8 as `utf8.RuneError`, for example the byte 0xFE that starts Go
// internal symbol names (`INTERNAL_SYMBOL_NAME_PREFIX`). The helpers below
// read the units of the port form: `decode_rune_in_string`, `go_runes`
// (Go `range`) and `go_map_runes` (Go `strings.Map`), which the crate
// prelude exports from `scanner_util`.

// Go: ls/autoimport/index.go:12 Named
// Named is a constraint for types that can provide their name.
pub trait Named {
    fn name(&self) -> String;
}

impl Named for Rc<Export> {
    fn name(&self) -> String {
        Export::name(self)
    }
}

// Go: ls/autoimport/index.go:19 Index
// Index stores entries with an index mapping uppercase letters to entries whose name
// starts with that letter, and lowercase letters to entries whose name contains a
// word starting with that letter.
// PORT: Go `map[rune][]int` is `FxHashMap<char, Vec<i32>>`; a Go nil map is
// an empty map. The owning bucket holds a shared index as
// `Rc<RefCell<Index<Rc<Export>>>>` (see `RegistryBucket`).
#[derive(Clone, Debug)]
pub struct Index<T: Named> {
    pub entries: Vec<T>,
    pub index: FxHashMap<char, Vec<i32>>,
}

impl<T: Named> Default for Index<T> {
    fn default() -> Self {
        Index {
            entries: Vec::new(),
            index: FxHashMap::default(),
        }
    }
}

impl<T: Named + Clone> Index<T> {
    // Go: ls/autoimport/index.go:24 Find
    // PORT: Go returns a nil slice for no results; that is an empty `Vec`.

    // Go: ls/autoimport/index.go:54 SearchWordPrefix
    // SearchWordPrefix returns each entry whose name contains a word beginning with
    // the first character of 'prefix', and whose name contains all characters
    // of 'prefix' in order (case-insensitive). If 'filter' is provided, only entries
    // for which filter(entry) returns true are included.
    // PORT: Go returns the shared `idx.entries` slice for an empty prefix; the
    // port returns a copy. Go nil results are an empty `Vec`.

    // Go: ls/autoimport/index.go:114 insertAsWords
    // insertAsWords adds a value to the index keyed by the first letter of each word in its name.

    // Go: ls/autoimport/index.go:152 Clone
    // Clone creates a new Index containing only entries for which filter returns true.
    // PORT: Go allows a nil receiver (`idx == nil` returns nil); `idx` is
    // `None` for it, and the nil result is `None`. `clone_` because
    // `Clone::clone` is taken.
    pub fn clone_(idx: Option<&Index<T>>, filter: &mut dyn FnMut(&T) -> bool) -> Option<Index<T>> {
        let idx = idx?;

        let mut new_idx = Index {
            entries: Vec::with_capacity(idx.entries.len()),
            index: FxHashMap::default(),
        };
        new_idx.index.reserve(idx.index.len());

        // Build mapping from old index to new index for filtered entries
        let mut old_to_new: FxHashMap<i32, i32> = FxHashMap::default();
        old_to_new.reserve(idx.entries.len());
        for (old_index, entry) in idx.entries.iter().enumerate() {
            if filter(entry) {
                let new_index = new_idx.entries.len() as i32;
                new_idx.entries.push(entry.clone());
                old_to_new.insert(old_index as i32, new_index);
            }
        }

        // Rebuild the index with remapped indices
        for (r, old_indices) in &idx.index {
            let mut new_indices: Vec<i32> = Vec::with_capacity(old_indices.len());
            for old_index in old_indices {
                if let Some(&new_index) = old_to_new.get(old_index) {
                    new_indices.push(new_index);
                }
            }
            if !new_indices.is_empty() {
                new_idx.index.insert(*r, new_indices);
            }
        }

        Some(new_idx)
    }
}

// Go: ls/autoimport/index.go:97 containsCharsInOrder
// containsCharsInOrder checks if str contains all characters from pattern in order (case-insensitive).


