use crate::frontend::prelude::*;
use std::sync::LazyLock;

// PORT: Go `core.WatchFileKind`, `core.WatchDirectoryKind` and
// `core.PollingKind` (core/watchoptions.go) are not in `crate::flags` and no
// unit owns core/watchoptions.go. They are defined here with the Go values
// because the enum maps below need them.
// Go: core/watchoptions.go:15 WatchFileKind
crate::flags_macros::go_enum!(WatchFileKind, i32 {
    NONE = 0; // WatchFileKindNone
    FIXED_POLLING_INTERVAL = 1; // WatchFileKindFixedPollingInterval
    PRIORITY_POLLING_INTERVAL = 2; // WatchFileKindPriorityPollingInterval
    DYNAMIC_PRIORITY_POLLING = 3; // WatchFileKindDynamicPriorityPolling
    FIXED_CHUNK_SIZE_POLLING = 4; // WatchFileKindFixedChunkSizePolling
    USE_FS_EVENTS = 5; // WatchFileKindUseFsEvents
    USE_FS_EVENTS_ON_PARENT_DIRECTORY = 6; // WatchFileKindUseFsEventsOnParentDirectory
});

// Go: core/watchoptions.go:27 WatchDirectoryKind
crate::flags_macros::go_enum!(WatchDirectoryKind, i32 {
    NONE = 0; // WatchDirectoryKindNone
    USE_FS_EVENTS = 1; // WatchDirectoryKindUseFsEvents
    FIXED_POLLING_INTERVAL = 2; // WatchDirectoryKindFixedPollingInterval
    DYNAMIC_PRIORITY_POLLING = 3; // WatchDirectoryKindDynamicPriorityPolling
    FIXED_CHUNK_SIZE_POLLING = 4; // WatchDirectoryKindFixedChunkSizePolling
});

// Go: core/watchoptions.go:37 PollingKind
crate::flags_macros::go_enum!(PollingKind, i32 {
    NONE = 0; // PollingKindNone
    FIXED_INTERVAL = 1; // PollingKindFixedInterval
    PRIORITY_INTERVAL = 2; // PollingKindPriorityInterval
    DYNAMIC_PRIORITY = 3; // PollingKindDynamicPriority
    FIXED_CHUNK_SIZE = 4; // PollingKindFixedChunkSize
});


// Go: tsoptions/enummaps.go:11 LibMap
// PORT: Go package-level vars built at init are `LazyLock` statics. The Go
// `any` values are `CompilerOptionsValue`. LibMap values are Go strings.

// Go: tsoptions/enummaps.go:136 Libs

// Go: tsoptions/enummaps.go:137 LibFilesSet

// Go: tsoptions/enummaps.go:140 GetLibFileName


// Go: tsoptions/enummaps.go:153 moduleResolutionOptionMap

// Go: tsoptions/enummaps.go:162 targetOptionMap

// Go: tsoptions/enummaps.go:180 moduleOptionMap

// Go: tsoptions/enummaps.go:197 moduleDetectionOptionMap

// Go: tsoptions/enummaps.go:203 jsxOptionMap

// Go: tsoptions/enummaps.go:211 newLineOptionMap

// Go: tsoptions/enummaps.go:216 targetToLibMap
pub static TARGET_TO_LIB_MAP: LazyLock<FxHashMap<ScriptTarget, String>> = LazyLock::new(|| {
    let entries: [(ScriptTarget, &str); 13] = [
        (ScriptTarget::ES_NEXT, "lib.esnext.full.d.ts"),
        (ScriptTarget::ES2026, "lib.es2026.full.d.ts"),
        (ScriptTarget::ES2025, "lib.es2025.full.d.ts"),
        (ScriptTarget::ES2024, "lib.es2024.full.d.ts"),
        (ScriptTarget::ES2023, "lib.es2023.full.d.ts"),
        (ScriptTarget::ES2022, "lib.es2022.full.d.ts"),
        (ScriptTarget::ES2021, "lib.es2021.full.d.ts"),
        (ScriptTarget::ES2020, "lib.es2020.full.d.ts"),
        (ScriptTarget::ES2019, "lib.es2019.full.d.ts"),
        (ScriptTarget::ES2018, "lib.es2018.full.d.ts"),
        (ScriptTarget::ES2017, "lib.es2017.full.d.ts"),
        (ScriptTarget::ES2016, "lib.es2016.full.d.ts"),
        (ScriptTarget::ES2015, "lib.es6.d.ts"), // We don't use lib.es2015.full.d.ts due to breaking change.
    ];
    entries
        .into_iter()
        .map(|(t, s)| (t, s.to_string()))
        .collect()
});

// Go: tsoptions/enummaps.go:232 TargetToLibMap
#[must_use]
pub fn target_to_lib_map() -> &'static FxHashMap<ScriptTarget, String> {
    &TARGET_TO_LIB_MAP
}

// Go: tsoptions/enummaps.go:236 GetDefaultLibFileName

// Go: tsoptions/enummaps.go:244 watchFileEnumMap

// Go: tsoptions/enummaps.go:253 watchDirectoryEnumMap

// Go: tsoptions/enummaps.go:260 fallbackEnumMap
