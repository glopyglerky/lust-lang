//! Version and provenance checks for inspection data. This is deliberately not
//! an executable-IR validator, an authenticity check, or a complete cache key.
use serde_json::{Map, Value};

pub const SCHEMA: &str = "lustr.rustc-semantic-inspection";
pub const VERSION: u64 = 1;

pub fn validate_envelope(report: &Value) -> Result<(), String> {
    object(
        report,
        &[
            "schema",
            "schema_version",
            "purpose",
            "executable_ir",
            "cache_key_complete",
            "compilation_succeeded",
            "compiler",
            "invocation",
            "crate",
            "configuration",
            "sources",
            "dependencies",
            "codegen_units",
        ],
        "report",
    )?;
    equal(&report["schema"], Value::from(SCHEMA), "schema")?;
    equal(&report["schema_version"], Value::from(VERSION), "schema_version")?;
    equal(&report["purpose"], Value::from("inspection-only"), "purpose")?;
    equal(&report["executable_ir"], Value::Bool(false), "executable_ir")?;
    equal(&report["cache_key_complete"], Value::Bool(false), "cache_key_complete")?;
    equal(&report["compilation_succeeded"], Value::Bool(true), "compilation_succeeded")?;

    object(&report["compiler"], &["commit", "host", "release", "sysroot"], "compiler")?;
    for (field, expected) in [("commit", crate::COMMIT), ("host", crate::HOST), ("release", crate::RELEASE), ("sysroot", crate::SYSROOT)] {
        equal(&report["compiler"][field], Value::from(expected), &format!("compiler.{field}"))?;
    }

    object(&report["invocation"], &["compiler", "arguments", "working_directory"], "invocation")?;
    nonempty_string(&report["invocation"]["compiler"], "invocation.compiler")?;
    let arguments = report["invocation"]["arguments"].as_array().ok_or("invocation.arguments must be an array")?;
    if arguments.iter().any(|argument| !argument.is_string()) {
        return Err("invocation.arguments must contain only strings".into());
    }
    let working_directory = &report["invocation"]["working_directory"];
    if !working_directory.is_string() && !working_directory.is_object() {
        return Err("invocation.working_directory must be an OS path representation".into());
    }

    let configuration = &report["configuration"];
    object(
        configuration,
        &["target", "data_layout", "panic_strategy", "opt_level", "debug_assertions", "overflow_checks", "mode"],
        "configuration",
    )?;
    for field in ["target", "data_layout", "opt_level"] {
        nonempty_string(&configuration[field], &format!("configuration.{field}"))?;
    }
    for field in ["debug_assertions", "overflow_checks"] {
        if !configuration[field].is_boolean() {
            return Err(format!("configuration.{field} must be boolean"));
        }
    }
    if !matches!(configuration["panic_strategy"].as_str(), Some("Unwind" | "Abort")) {
        return Err("configuration.panic_strategy must be Unwind or Abort".into());
    }
    for field in ["sources", "dependencies", "codegen_units"] {
        if !report[field].is_array() {
            return Err(format!("{field} must be an array"));
        }
    }
    match configuration["mode"].as_str() {
        Some("analysis-only") if report["codegen_units"].as_array().unwrap().is_empty() => {}
        Some("codegen") => {}
        _ => return Err("invalid configuration.mode or codegen units in analysis-only report".into()),
    }
    object(&report["crate"], &["name", "stable_id", "crate_hash", "crate_hash_status", "types"], "crate")?;
    nonempty_string(&report["crate"]["name"], "crate.name")?;
    nonempty_string(&report["crate"]["stable_id"], "crate.stable_id")?;
    equal(&report["crate"]["crate_hash"], Value::Null, "crate.crate_hash")?;
    equal(&report["crate"]["crate_hash_status"], Value::from("not-available-after-analysis"), "crate.crate_hash_status")?;
    let types = report["crate"]["types"].as_array().ok_or("crate.types must be an array")?;
    if types.is_empty() || types.iter().any(|kind| !kind.is_string()) {
        return Err("crate.types must be a nonempty string array".into());
    }
    Ok(())
}

fn object<'a>(value: &'a Value, fields: &[&str], name: &str) -> Result<&'a Map<String, Value>, String> {
    let value = value.as_object().ok_or_else(|| format!("{name} must be an object"))?;
    for field in fields {
        if !value.contains_key(*field) {
            return Err(format!("{name} is missing required field {field}"));
        }
    }
    if let Some(field) = value.keys().find(|field| !fields.contains(&field.as_str())) {
        return Err(format!("{name} contains an unknown field {field}; schema version must change"));
    }
    Ok(value)
}

fn equal(actual: &Value, expected: Value, name: &str) -> Result<(), String> {
    if actual != &expected {
        return Err(format!("{name} mismatch: expected {expected}, got {actual}"));
    }
    Ok(())
}

fn nonempty_string(value: &Value, name: &str) -> Result<(), String> {
    if !value.as_str().is_some_and(|value| !value.is_empty()) {
        return Err(format!("{name} must be a nonempty string"));
    }
    Ok(())
}
