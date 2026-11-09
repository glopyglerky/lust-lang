//! Go `internal/project/ata/typesmap.go`.
//!
//! PORT: Go `map[string]string` literal. Go map iteration order is random;
//! the Rust map keeps the Go source order (`IndexMap`). Go code only looks
//! keys up, so the order never reaches output.

use crate::project::ata::prelude::*;
use std::sync::LazyLock;

// type safeListEntry struct {
// 	match   string
// 	exclude []any
// 	types   string
// }

// var typesMap = map[string]safeListEntry{
// 	"jquery": {
// 		match: `jquery(-(\\.?\\d+)+)?(\\.intellisense)?(\\.min)?\\.js$`,
// 		types: "jquery",
// 	},
// 	"WinJS": {
// 		match:   `^(.*\\/winjs-[.\\d]+)\\/js\\/base\\.js$`,
// 		exclude: []any{"^", 1, "/.*"},
// 		types:   "winjs",
// 	},
// 	"Kendo": {
// 		match:   `^(.*\\/kendo(-ui)?)\\/kendo\\.all(\\.min)?\\.js$`,
// 		exclude: []any{"^", 1, "/.*"},
// 		types:   "kendo-ui",
// 	},
// 	"Office Nuget": {
// 		match:   `^(.*\\/office\\/1)\\/excel-\\d+\\.debug\\.js$`,
// 		exclude: []any{"^", 1, "/.*"},
// 		types:   "office",
// 	},
// 	"References": {
// 		match:   `^(.*\\/_references\\.js)$`,
// 		exclude: []any{"^", 1, "$"},
// 		types:   "",
// 	},
// 	"Datatables.net": {
// 		match: `^.*\\/(jquery\\.)?dataTables(\\.all)?(\\.min)?\\.js$`,
// 		types: "datatables.net",
// 	},
// 	"Ace": {
// 		match:   `^(.*)\\/ace.js`,
// 		exclude: []any{"^", 1, "/.*"},
// 		types:   "ace",
// 	},
// }

// Go: project/ata/typesmap.go:45 safeFileNameToTypeName
