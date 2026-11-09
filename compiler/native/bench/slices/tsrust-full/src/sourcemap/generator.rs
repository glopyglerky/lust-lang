//! Port of Go `sourcemap/generator.go`: the source map generator that the
//! printer feeds while it writes a file.

use crate::prelude::*;

use crate::frontend::tspath::{ComparePathsOptions, };

/// Go `sourcemap.SourceIndex`.
pub type SourceIndex = i32;
/// Go `sourcemap.NameIndex`.
pub type NameIndex = i32;

const SOURCE_INDEX_NOT_SET: SourceIndex = -1;
const NAME_INDEX_NOT_SET: NameIndex = -1;
const NOT_SET: i32 = -1;
const NOT_SET_UTF16: i32 = -1;

// Go: sourcemap/generator.go:26 Generator
// PORT: Go `int` lines and `core.UTF16Offset` characters are `i32`, like the
// printer writer. Go `[]*string` sources content is `Vec<Option<String>>`.
#[derive(Debug, Default)]
pub struct Generator {
    path_options: ComparePathsOptions,
    file: String,
    source_root: String,
    sources_directory_path: String,
    raw_sources: Vec<String>,
    sources: Vec<String>,
    source_to_source_index_map: FxHashMap<String, SourceIndex>,
    sources_content: Vec<Option<String>>,
    names: Vec<String>,
    name_to_name_index_map: FxHashMap<String, NameIndex>,
    mappings: String,
    last_generated_line: i32,
    last_generated_character: i32,
    last_source_index: SourceIndex,
    last_source_line: i32,
    last_source_character: i32,
    last_name_index: NameIndex,
    has_last: bool,
    pending_generated_line: i32,
    pending_generated_character: i32,
    pending_source_index: SourceIndex,
    pending_source_line: i32,
    pending_source_character: i32,
    pending_name_index: NameIndex,
    has_pending: bool,
    has_pending_source: bool,
    has_pending_name: bool,
}

// Go: sourcemap/generator.go:55 RawSourceMap
// PORT: Go `SourcesContent []*string` with `omitzero` is `None` when Go has a
// nil slice.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RawSourceMap {
    pub version: i32,
    pub file: String,
    pub source_root: String,
    pub sources: Vec<String>,
    pub names: Vec<String>,
    pub mappings: String,
    pub sources_content: Option<Vec<Option<String>>>,
}

impl RawSourceMap {
}

/// Borrowed `RawSourceMap` fields. `RawSourceMap::to_json` and
/// `Generator::bytes` both write JSON through it, so the key order and the
/// omit rules are in one place and the generator clones nothing.
struct RawSourceMapJson<'a> {
    version: i32,
    file: &'a str,
    source_root: &'a str,
    sources: &'a [String],
    names: &'a [String],
    mappings: &'a str,
    /// `None` omits the key (Go `omitzero` on a nil slice).
    sources_content: Option<&'a [Option<String>]>,
}

impl RawSourceMapJson<'_> {

    /// The output size without escapes, so the common case never grows the
    /// buffer. Each list item adds two quotes and a comma.
    fn len_hint(&self) -> usize {
        let list = |values: &[String]| values.iter().map(|v| v.len() + 3).sum::<usize>();
        let content = self.sources_content.map_or(0, |contents| {
            contents
                .iter()
                .map(|c| c.as_ref().map_or(5, |c| c.len() + 3))
                .sum::<usize>()
        });
        128 + self.file.len()
            + self.source_root.len()
            + self.mappings.len()
            + list(self.sources)
            + list(self.names)
            + content
    }
}




// Go: sourcemap/generator.go:66 NewGenerator
#[must_use]
pub fn new_generator(
    file: &str,
    source_root: &str,
    sources_directory_path: &str,
    options: ComparePathsOptions,
) -> Generator {
    Generator {
        file: file.to_string(),
        source_root: source_root.to_string(),
        sources_directory_path: sources_directory_path.to_string(),
        path_options: options,
        // PORT: skips the first small regrowths of the mappings buffer.
        mappings: String::with_capacity(MAPPINGS_INITIAL_CAPACITY),
        ..Generator::default()
    }
}

/// Start size of the mappings buffer, so small files never regrow it.
const MAPPINGS_INITIAL_CAPACITY: usize = 1024;

impl Generator {
    // Go: sourcemap/generator.go:75 Sources
    #[must_use]
    pub fn sources(&self) -> Vec<String> {
        self.raw_sources.clone()
    }

    // Go: sourcemap/generator.go:78 AddSource
    // Adds a source to the source map

    // Go: sourcemap/generator.go:101 SetSourceContent
    // Sets the content for a source
    pub fn set_source_content(
        &mut self,
        source_index: SourceIndex,
        content: &str,
    ) -> Result<(), String> {
        if source_index < 0 || source_index as usize >= self.sources.len() {
            return Err("sourceIndex is out of range".to_string());
        }
        while self.sources_content.len() <= source_index as usize {
            self.sources_content.push(None);
        }
        self.sources_content[source_index as usize] = Some(content.to_string());
        Ok(())
    }

    // Go: sourcemap/generator.go:113 AddName
    // Declares a name in the source map, returning the index of the name
    pub fn add_name(&mut self, name: &str) -> NameIndex {
        if let Some(&name_index) = self.name_to_name_index_map.get(name) {
            return name_index;
        }
        let name_index = self.names.len() as NameIndex;
        self.names.push(name.to_string());
        self.name_to_name_index_map
            .insert(name.to_string(), name_index);
        name_index
    }

    // Go: sourcemap/generator.go:126 isNewGeneratedPosition
    fn is_new_generated_position(&self, generated_line: i32, generated_character: i32) -> bool {
        !self.has_pending
            || self.pending_generated_line != generated_line
            || self.pending_generated_character != generated_character
    }

    // Go: sourcemap/generator.go:132 isBacktrackingSourcePosition
    fn is_backtracking_source_position(
        &self,
        source_index: SourceIndex,
        source_line: i32,
        source_character: i32,
    ) -> bool {
        source_index != SOURCE_INDEX_NOT_SET
            && source_line != NOT_SET
            && source_character != NOT_SET_UTF16
            && self.pending_source_index == source_index
            && (self.pending_source_line > source_line
                || self.pending_source_line == source_line
                    && self.pending_source_character > source_character)
    }

    // Go: sourcemap/generator.go:141 shouldCommitMapping
    fn should_commit_mapping(&self) -> bool {
        self.has_pending
            && (!self.has_last
                || self.last_generated_line != self.pending_generated_line
                || self.last_generated_character != self.pending_generated_character
                || self.last_source_index != self.pending_source_index
                || self.last_source_line != self.pending_source_line
                || self.last_source_character != self.pending_source_character
                || self.last_name_index != self.pending_name_index)
    }

    // Go: sourcemap/generator.go:151 appendMappingCharCode
    // PORT: takes an ASCII byte. The `& 0x7F` changes no caller's value; it
    // tells the compiler the char is ASCII, so `push` writes one byte.

    // Go: sourcemap/generator.go:155 appendBase64VLQ

    // Go: sourcemap/generator.go:182 commitPendingMapping

    // Go: sourcemap/generator.go:237 addMapping

    // Go: sourcemap/generator.go:261 AddGeneratedMapping
    // Adds a mapping without source information

    // Go: sourcemap/generator.go:275 AddSourceMapping
    // Adds a mapping with source information

    // Go: sourcemap/generator.go:299 AddNamedSourceMapping
    // Adds a mapping with source and name information

    // Go: sourcemap/generator.go:316 RawSourceMap
    // Gets the source map as a `RawSourceMap` object

    // Go: sourcemap/generator.go:347 bytes
    // PORT: Go marshals the cloned `RawSourceMap()`. This writes the same
    // JSON from the fields in place, after the same commit.

    // Go: sourcemap/generator.go:356 String
    // Gets the string representation of the source map

    // Go: sourcemap/generator.go:360 Base64DataURL
}

/// The standard base64 digits. Both `base64FormatEncode` and
/// `base64.StdEncoding` use this order.
const BASE64_DIGITS: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";


// Go: sourcemap/generator.go:372 base64FormatEncode
// PORT: a table lookup that returns the ASCII byte. A value past 63 panics
// on the index, as Go panics.
fn base64_format_encode(value: usize) -> u8 {
    BASE64_DIGITS[value]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vlq_and_json_match_go() {
        let mut generator = new_generator("a.js", "", "/p", ComparePathsOptions::default());
        let source = generator.add_source("/p/a.ts");
        generator.add_source_mapping(0, 0, source, 0, 0).unwrap();
        generator.add_source_mapping(0, 4, source, 0, 6).unwrap();
        generator.add_source_mapping(1, 0, source, 1, 0).unwrap();
        assert_eq!(
            generator.string(),
            r#"{"version":3,"file":"a.js","sourceRoot":"","sources":["a.ts"],"names":[],"mappings":"AAAA,IAAM;AACN"}"#
        );
    }

    #[test]
    fn generator_json_matches_raw_source_map_json() {
        let mut generator = new_generator("a.js", "r/", "/p", ComparePathsOptions::default());
        let source = generator.add_source("/p/a.ts");
        generator
            .set_source_content(source, "let x = \"\\\n")
            .unwrap();
        let name = generator.add_name("x");
        generator
            .add_named_source_mapping(0, 4, source, 0, 4, name)
            .unwrap();
        generator.add_source_mapping(2, 0, source, 1, 0).unwrap();
        let raw = generator.raw_source_map().to_json();
        assert_eq!(generator.string(), raw);
        assert_eq!(
            raw,
            r#"{"version":3,"file":"a.js","sourceRoot":"r/","sources":["a.ts"],"names":["x"],"mappings":"IAAIA;;AACJ","sourcesContent":["let x = \"\\\n"]}"#
        );

        // A decoded map can hold mappings that need escapes.
        let decoded = RawSourceMap {
            version: 3,
            mappings: "A\"".to_string(),
            ..RawSourceMap::default()
        };
        assert_eq!(
            decoded.to_json(),
            r#"{"version":3,"file":"","sourceRoot":"","sources":[],"names":[],"mappings":"A\""}"#
        );
    }

    #[test]
    fn base64_padding() {
        let mut s = String::new();
        base64_std_encode(&mut s, b"ab");
        assert_eq!(s, "YWI=");
    }
}
