//! Bench driver for the slices (bench text, not ts-rust code). The benchmark
//! mounts it as `mod bench_driver;` and calls `run()` from `main`, so the build
//! rows can run the program and compare its output with rustc's binary. It
//! calls ts-rust tspath functions that every slice keeps.

use crate::frontend::tspath::path::{
    contains_ignored_path, ensure_trailing_directory_separator, file_extension_is, has_trailing_directory_separator,
    is_dynamic_file_name, path_is_relative, remove_trailing_directory_separator, remove_trailing_directory_separators,
};

pub fn run() {
    let paths = [
        "/a/b/",
        "a/b",
        "./x.ts",
        "../up/y.d.ts",
        "^/dynamic",
        "c:\\dir\\",
        "/repo/node_modules/pkg/index.d.ts",
        "/repo/.git/HEAD",
        "///",
        "",
    ];
    for p in paths {
        println!(
            "{:?}: dynamic={} trailing={} relative={} ts={} ignored={} trimmed={:?} trimmed_all={:?} ensured={:?}",
            p,
            is_dynamic_file_name(p),
            has_trailing_directory_separator(p),
            path_is_relative(p),
            file_extension_is(p, ".ts"),
            contains_ignored_path(p),
            remove_trailing_directory_separator(p),
            remove_trailing_directory_separators(p),
            ensure_trailing_directory_separator(p),
        );
    }
}
