// macro: files, file_map, args
// source: crates/ts_goport/tests/go_baselines/tsctests/tsbuild_a.rs:24
// source: crates/ts_goport/tests/go_baselines/support/runner.rs:43
// source: crates/ts_goport/tests/go_baselines/support/runner.rs:59
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
pub mod support {
    pub mod vfstest {
        #[derive(Clone, Debug, PartialEq)]
        pub struct MapFile(pub String);
        impl From<&str> for MapFile { fn from(s: &str) -> Self { MapFile(s.to_string()) } }
        impl From<String> for MapFile { fn from(s: String) -> Self { MapFile(s) } }
    }
    pub mod runner {
        pub type FileMap = std::collections::BTreeMap<String, super::vfstest::MapFile>;
    }
}
use support::runner::FileMap;
use support::vfstest::MapFile;

macro_rules! files {
    ($($path:expr => $value:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut files = FileMap::new();
        $(files.insert(String::from($path), MapFile::from($value));)*
        files
    }};
}

macro_rules! file_map {
    ($($path:expr => $value:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut files = $crate::support::runner::FileMap::new();
        $(
            files.insert(
                ::std::string::String::from($path),
                $crate::support::vfstest::MapFile::from($value),
            );
        )*
        files
    }};
}

macro_rules! args {
    ($($arg:expr),* $(,)?) => {
        ::std::vec![$(::std::string::String::from($arg)),*]
    };
}

fn main() {
    let a = files!["/src/a.ts" => "aaa", "/src/b.ts" => "bbb"];
    let keys: Vec<&String> = a.keys().collect();
    println!("files={} keys={keys:?}", a.len());
    let b = file_map!["/x.ts" => "x"];
    println!("file_map={} first={}", b.len(), b.values().next().unwrap().0);
    let argv: Vec<String> = args!["--build", "proj"];
    println!("args={argv:?}");
}
