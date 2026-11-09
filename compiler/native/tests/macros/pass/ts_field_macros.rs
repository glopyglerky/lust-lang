// macro: merge_fields, substitute_string_fields, take
// source: crates/ts_goport/src/frontend/tsoptions/parsing_helpers.rs:738
// source: crates/ts_goport/src/frontend/tsoptions/tsconfig_p2.rs:1654
// source: crates/ts_goport/src/effect/configraw.rs:54
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
use std::collections::HashSet;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Opts { pub out_dir: String, pub strict: bool, pub target: String }
fn is_zero_value<T: Default + PartialEq>(v: &T) -> bool { *v == T::default() }

pub enum CompilerOptionsValue { String(String) }
fn starts_with_config_dir_template(v: &CompilerOptionsValue) -> bool {
    matches!(v, CompilerOptionsValue::String(s) if s.starts_with("${configDir}"))
}
fn get_substituted_path_with_config_dir_template(s: &str, base_path: &str) -> String {
    s.replacen("${configDir}", base_path, 1)
}
#[derive(Clone, Default, Debug)]
pub struct RawOpts { pub root_dir: String, pub out_file: String, pub other: String }

fn merge(target_options: &mut Opts, source_options: &Opts, explicit_null_fields: &HashSet<String>) {
    macro_rules! merge_fields {
        ($($field:ident => $json:literal,)*) => {
            $(
                // Get the JSON field name for this struct field and check if it's explicitly null
                if explicit_null_fields.contains($json) {
                    target_options.$field = Default::default();
                } else if !is_zero_value(&source_options.$field) {
                    // Normal merge behavior: copy non-zero fields
                    target_options.$field = source_options.$field.clone();
                }
            )*
        };
    }
    merge_fields! {
        out_dir => "outDir",
        strict => "strict",
        target => "target",
    }
}

fn substitute(compiler_options: &mut RawOpts, base_path: &str) {
    macro_rules! substitute_string_fields {
        ($($field:ident),*) => {
            $(
                if starts_with_config_dir_template(&CompilerOptionsValue::String(compiler_options.$field.clone())) {
                    compiler_options.$field = get_substituted_path_with_config_dir_template(&compiler_options.$field, base_path);
                }
            )*
        };
    }
    substitute_string_fields!(root_dir, out_file);
}

fn take_from(source: &RawOpts, merged: &mut RawOpts, keys: &[&str]) {
    let has = |k: &str| keys.contains(&k);
    macro_rules! take {
        ($($key:literal => $field:ident,)*) => {
            $(if has($key) {
                merged.$field = source.$field.clone();
            })*
        };
    }
    take! {
        "rootDir" => root_dir,
        "outFile" => out_file,
    }
}

fn main() {
    let source = Opts { out_dir: "dist".into(), strict: true, target: String::new() };
    let mut target = Opts { out_dir: "keep".into(), strict: false, target: "es2022".into() };
    let nulls: HashSet<String> = ["outDir".to_string()].into_iter().collect();
    merge(&mut target, &source, &nulls);
    // out_dir is explicit null so it resets; strict copies; empty target does not copy.
    println!("merged={target:?}");
    let mut raw = RawOpts { root_dir: "${configDir}/src".into(), out_file: "plain.js".into(), other: String::new() };
    substitute(&mut raw, "/proj");
    println!("subst={} {}", raw.root_dir, raw.out_file);
    let mut merged = RawOpts::default();
    take_from(&raw, &mut merged, &["rootDir"]);
    println!("take={} '{}'", merged.root_dir, merged.out_file);
}
