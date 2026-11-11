//! Recompute native requests whose external inputs the pinned response cache
//! cannot track. This is an invocation policy, not a compiler/source patch or a
//! claim that the remaining native cache is qualified for arbitrary Rust.
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;

const MAX_SOURCE_BYTES: u64 = 4 * 1024 * 1024;

pub(crate) fn protect(command: &mut Command, args: &[OsString]) {
    if must_recompute(args) {
        // A daemon already running with caching enabled may not honor this
        // request's cache setting. A fresh native process must own the setting.
        command.env("LUST_NO_CACHE", "1").env("LUST_NO_DAEMON", "1");
    }
}

fn must_recompute(args: &[OsString]) -> bool {
    if !matches!(args.first().and_then(|arg| arg.to_str()), Some("check" | "build" | "run")) {
        return false;
    }
    // Only the documented command + source form identifies the input here.
    // Do not duplicate Lust's option parser or alter its diagnostics. If flags
    // precede the source, let Lust resolve them with request reuse disabled.
    let Some(path) = args.get(1).filter(|arg| !arg.as_encoded_bytes().starts_with(b"-")) else {
        return true;
    };
    read_bounded(Path::new(path)).is_none_or(|source| mentions_external_input(&source))
}

fn read_bounded(path: &Path) -> Option<Vec<u8>> {
    // Never consume stdin or a device as a preflight. Lust remains responsible
    // for interpreting an unavailable input and reporting its own failure.
    if !path.metadata().ok()?.is_file() {
        return None;
    }
    let file = File::open(path).ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    let mut source = Vec::new();
    file.take(MAX_SOURCE_BYTES + 1).read_to_end(&mut source).ok()?;
    (source.len() as u64 <= MAX_SOURCE_BYTES).then_some(source)
}

fn mentions_external_input(source: &[u8]) -> bool {
    // The upstream cache looks for literal `env!`/`include*!` byte sequences.
    // Rust allows whitespace and nested comments before `!`. Seeing a complete
    // name is sufficient to refuse reuse regardless of the following gap.
    // Deliberately also match names in strings/comments: an extra cold build is
    // safe, whereas guessing whether a token is semantically active is not.
    source
        .split(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
        .any(|word| matches!(word, b"env" | b"option_env" | b"include" | b"include_str" | b"include_bytes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_input_names_refuse_reuse_with_every_macro_gap() {
        for name in ["env", "option_env", "include", "include_str", "include_bytes"] {
            for gap in ["", " ", "\n\t", " /* gap */ ", " /* outer /* inner */ end */ ", " // gap\n", "\u{200e}"] {
                let source = format!("fn main() {{ {name}{gap}!(\"input\"); }}");
                assert!(mentions_external_input(source.as_bytes()), "{source}");
            }
        }
    }

    #[test]
    fn plain_names_keep_reuse_and_uncertain_contexts_recompute() {
        assert!(!mentions_external_input(b"fn main() { let environment = 42; println!(\"{}\", environment); }"));
        assert!(mentions_external_input(b"// env may be emitted by an expansion"));
        assert!(mentions_external_input(br#"const NAME: &str = "include_str";"#));
    }
}
