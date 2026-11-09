// K1 compatibility fixture: self: &Rc<Self> methods.
// panic-mode: abort
//
// Real shape: crates/ts_goport/src/frontend/packagejson.rs
// InfoCacheEntry::with_package_directory (verbatim below): an
// &Rc<Self> receiver returns the same Rc when no change is needed,
// else a corrected shallow copy in a new Rc. The LSP server in
// ts-rust uses &Rc<Self> receivers throughout (lsp/server.rs).

use std::rc::Rc;

/// A cached package.json read (fields reduced to the used ones).
#[derive(Clone, Debug)]
struct InfoCacheEntry {
    package_directory: String,
    directory_exists: bool,
    contents: Option<String>,
}

impl InfoCacheEntry {
    // Because the cache uses first-writer-wins semantics, a later
    // caller may receive an entry whose package_directory does not
    // match its own candidate path, so return a corrected shallow
    // copy when they diverge. Verbatim from ts-rust packagejson.rs.
    #[must_use]
    pub fn with_package_directory(self: &Rc<Self>, package_directory: &str) -> Rc<InfoCacheEntry> {
        if self.package_directory == package_directory {
            return self.clone();
        }
        Rc::new(InfoCacheEntry {
            package_directory: package_directory.to_string(),
            directory_exists: self.directory_exists,
            contents: self.contents.clone(),
        })
    }

    /// A derived entry that keeps identity when possible is the
    /// second common shape: compare a field, return self or a copy.
    #[must_use]
    pub fn with_contents(self: &Rc<Self>, contents: Option<String>) -> Rc<InfoCacheEntry> {
        if self.contents == contents {
            return self.clone();
        }
        Rc::new(InfoCacheEntry {
            package_directory: self.package_directory.clone(),
            directory_exists: self.directory_exists,
            contents,
        })
    }
}

fn main() {
    let entry = Rc::new(InfoCacheEntry {
        package_directory: "node_modules/preact".to_string(),
        directory_exists: true,
        contents: Some("{\"name\":\"preact\"}".to_string()),
    });
    println!("start refs: {}", Rc::strong_count(&entry));

    let same = entry.with_package_directory("node_modules/preact");
    println!(
        "same dir: ptr_eq={} refs={}",
        Rc::ptr_eq(&entry, &same),
        Rc::strong_count(&entry)
    );

    let moved = entry.with_package_directory("node_modules/preact/compat");
    println!(
        "moved dir: ptr_eq={} dir={} exists={} contents={:?}",
        Rc::ptr_eq(&entry, &moved),
        moved.package_directory,
        moved.directory_exists,
        moved.contents
    );
    println!("original unchanged: {}", entry.package_directory);

    let filled = moved.with_contents(Some("{\"name\":\"preact/compat\"}".to_string()));
    println!("filled: ptr_eq={} contents={:?}", Rc::ptr_eq(&moved, &filled), filled.contents);
    let refilled = filled.with_contents(Some("{\"name\":\"preact/compat\"}".to_string()));
    println!("refilled same: ptr_eq={}", Rc::ptr_eq(&filled, &refilled));

    // Method call through a plain Rc variable and through a slice
    // of entries, as cache lookups do.
    let cache: Vec<Rc<InfoCacheEntry>> = vec![entry.clone(), moved.clone()];
    for e in &cache {
        let fixed = e.with_package_directory("node_modules/preact");
        println!("cache entry {} -> same={}", e.package_directory, Rc::ptr_eq(e, &fixed));
    }
}
