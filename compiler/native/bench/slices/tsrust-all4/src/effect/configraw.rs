//! Port of Effect-TS/tsgo `internal/effectconfigraw`: merging the Effect
//! plugin options across `extends`. A child config replaces only the plugin
//! keys it sets. `diagnosticSeverity` merges by rule, and `overrides`
//! appends. Override globs are rewritten relative to the config that
//! declared them.

use crate::effect::etscore::{EFFECT_PLUGIN_NAME, };
use crate::frontend::tspath::path::{
    ComparePathsOptions, 
    
};
use crate::indexmap::IndexMap;
use std::sync::Arc;

// Go: effectconfigraw MergeEffectCompilerOptions


// Go: mergeSeverityMaps. Go nil stays None only when both are nil.
fn merge_severity_maps<V: Clone>(
    target: Option<&IndexMap<String, V>>,
    source: Option<&IndexMap<String, V>>,
) -> Option<IndexMap<String, V>> {
    let Some(source) = source else {
        return target.cloned();
    };
    let mut merged = target.cloned().unwrap_or_default();
    for (k, v) in source {
        merged.insert(k.clone(), v.clone());
    }
    Some(merged)
}




const CONFIG_DIR_TEMPLATE: &str = "${configDir}";

fn starts_with_config_dir_template(value: &str) -> bool {
    value
        .to_lowercase()
        .starts_with(&CONFIG_DIR_TEMPLATE.to_lowercase())
}
