//! `lust-port scan`: reports every Rust feature lust does not support yet
//! in one cargo package (or a whole workspace), grouped by root cause.
//! Exposed as `lust port scan` by a small hook in lust's client, which
//! execs this binary as a sibling.
//!
//!   lust-port scan --manifest-path <Cargo.toml> [-p pkg] [--target triple]
//!                  [--json | --markdown] [--summary]
//!                  [--features features.json] [--capabilities caps.json]
//!                  [--dependencies dependencies.json]
//!                  [--cargo-features a,b | --cargo-exclude-features a,b]
//!
//! Cargo features: `cfg(feature = ...)` is evaluated with each
//! package's effective dev features (project lust config, else the
//! package defaults; see devfeat.rs). The two `--cargo-*` flags
//! override the config for one scan: an exact list, or names removed
//! from each package's `default` list.
//!
//! The scanner is a full Rust parser (syn). It survives unparseable
//! files and keeps going. Exit status is 0 for a completed scan, even
//! when gaps are found: gaps are the output, not an error.

mod analyze;
mod cargo_meta;
mod cfg_eval;
mod devfeat;
mod features;
mod global;
mod report;
mod stats;
mod walk;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

const USAGE: &str = "\
usage: lust-port scan --manifest-path <Cargo.toml> [-p pkg] [--target triple] [--json | --markdown] [--summary]
              [--features features.json] [--capabilities caps.json] [--dependencies dependencies.json]
              [--keys-json keys.json] [--cargo-features a,b | --cargo-exclude-features a,b]
       lust-port --help";

struct Args {
    manifest: PathBuf,
    package: Option<String>,
    target: Option<String>,
    json: bool,
    markdown: bool,
    summary: bool,
    features: Option<PathBuf>,
    capabilities: Option<PathBuf>,
    dependencies: Option<PathBuf>,
    keys_json: Option<PathBuf>,
    cargo_features: Option<Vec<String>>,
    cargo_exclude_features: Option<Vec<String>>,
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut it = argv.iter();
    match it.next().map(|s| s.as_str()) {
        Some("scan") => {}
        Some("--help") | Some("-h") | None => return Err(USAGE.to_string()),
        Some(other) => return Err(format!("unknown command `{other}`\n{USAGE}")),
    }
    let mut a = Args {
        manifest: PathBuf::from("Cargo.toml"),
        package: None,
        target: None,
        json: false,
        markdown: false,
        summary: false,
        features: None,
        capabilities: None,
        dependencies: None,
        keys_json: None,
        cargo_features: None,
        cargo_exclude_features: None,
    };
    let split_feats = |v: &str| -> Vec<String> {
        v.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).map(str::to_string).collect()
    };
    while let Some(x) = it.next() {
        let take = |it: &mut std::slice::Iter<'_, String>| -> Result<String, String> {
            it.next().cloned().ok_or_else(|| format!("{x} needs a value"))
        };
        match x.as_str() {
            "--manifest-path" => a.manifest = PathBuf::from(take(&mut it)?),
            "-p" | "--package" => a.package = Some(take(&mut it)?),
            "--target" => a.target = Some(take(&mut it)?),
            "--json" => a.json = true,
            "--markdown" => a.markdown = true,
            "--summary" => a.summary = true,
            "--features" => a.features = Some(PathBuf::from(take(&mut it)?)),
            "--capabilities" => a.capabilities = Some(PathBuf::from(take(&mut it)?)),
            "--dependencies" => a.dependencies = Some(PathBuf::from(take(&mut it)?)),
            "--keys-json" => a.keys_json = Some(PathBuf::from(take(&mut it)?)),
            "--cargo-features" => a.cargo_features = Some(split_feats(&take(&mut it)?)),
            "--cargo-exclude-features" => a.cargo_exclude_features = Some(split_feats(&take(&mut it)?)),
            "--help" | "-h" => return Err(USAGE.to_string()),
            _ if x.starts_with("--manifest-path=") => a.manifest = PathBuf::from(&x["--manifest-path=".len()..]),
            _ if x.starts_with("--target=") => a.target = Some(x["--target=".len()..].to_string()),
            _ if x.starts_with("-p=") => a.package = Some(x[3..].to_string()),
            _ if x.starts_with("--cargo-features=") => a.cargo_features = Some(split_feats(&x["--cargo-features=".len()..])),
            _ if x.starts_with("--cargo-exclude-features=") => {
                a.cargo_exclude_features = Some(split_feats(&x["--cargo-exclude-features=".len()..]))
            }
            other => return Err(format!("unknown argument `{other}`\n{USAGE}")),
        }
    }
    if a.json && a.markdown {
        return Err("pass only one of --json and --markdown".to_string());
    }
    if a.cargo_features.is_some() && a.cargo_exclude_features.is_some() {
        return Err("pass only one of --cargo-features and --cargo-exclude-features".to_string());
    }
    Ok(a)
}

fn read_file(p: &Path) -> Result<String, String> {
    std::fs::read_to_string(p).map_err(|e| format!("cannot read {}: {e}", p.display()))
}

/// Writes every raw scanner key with its count and the feature it maps
/// to (null when no rule matches). A null feature means the coverage
/// curve treats the construct as supported, so this dump is how new
/// rules get found: sort the null rows by count and probe the top ones.
fn write_keys_json(p: &Path, stats: &stats::Stats, feats: &features::FeatureTable) -> Result<(), String> {
    let mut rows: Vec<serde_json::Value> = stats
        .map
        .iter()
        .map(|(key, e)| {
            let feat = feats.classify(key);
            let files: std::collections::HashSet<u32> = e.files.iter().flatten().copied().collect();
            serde_json::json!({
                "key": key,
                "count": e.count.iter().sum::<u64>(),
                "prod_count": e.count[walk::Cat::Prod as usize] + e.count[walk::Cat::Gen as usize],
                "files": files.len(),
                "feature": feat.map(|f| f.id.clone()),
                "status": feat.map(|f| f.status.clone()),
                "samples": e.samples,
            })
        })
        .collect();
    rows.sort_by(|a, b| b["count"].as_u64().cmp(&a["count"].as_u64()).then(a["key"].as_str().cmp(&b["key"].as_str())));
    let text = serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?;
    std::fs::write(p, text + "\n").map_err(|e| format!("cannot write {}: {e}", p.display()))?;
    eprintln!("lust-port: wrote {} keys to {}", rows.len(), p.display());
    Ok(())
}

fn run(args: Args) -> Result<String, String> {
    let t0 = Instant::now();
    let manifest = std::path::absolute(&args.manifest).map_err(|e| e.to_string())?;
    if !manifest.exists() {
        return Err(format!("manifest not found: {}", manifest.display()));
    }
    let meta = cargo_meta::load(&manifest)?;
    let packages = cargo_meta::select(&meta, &manifest, args.package.as_deref())?;
    let target = cfg_eval::TargetInfo::from_triple(args.target.as_deref());

    let mut feats = match &args.features {
        Some(p) => features::FeatureTable::parse(&read_file(p)?)?,
        None => features::FeatureTable::embedded()?,
    };
    if let Some(p) = &args.capabilities {
        let unknown = feats.apply_capabilities(&read_file(p)?)?;
        if !unknown.is_empty() {
            eprintln!("warning: capabilities file names unknown feature ids: {}", unknown.join(", "));
        }
    }
    let dep_table = match &args.dependencies {
        Some(p) => serde_json::from_str::<report::DepTable>(&read_file(p)?).map_err(|e| format!("bad dependencies file: {e}"))?,
        None => report::DepTable::embedded()?,
    };

    // Effective dev features per package (project lust config, else the
    // package defaults; CLI flags override). Every later cfg decision
    // in this scan uses these sets.
    let mut eff_by_id: std::collections::HashMap<String, std::sync::Arc<std::collections::BTreeSet<String>>> =
        std::collections::HashMap::new();
    let mut enabled_deps: std::collections::HashMap<String, HashSet<String>> = std::collections::HashMap::new();
    let mut cargo_features_report: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    for p in &meta.packages {
        let eff = devfeat::effective(
            p,
            &meta.workspace_metadata,
            &meta.workspace_root,
            args.cargo_features.as_deref(),
            args.cargo_exclude_features.as_deref(),
        );
        enabled_deps.insert(p.name.clone(), eff.deps);
        if packages.iter().any(|s| s.id == p.id) {
            cargo_features_report.insert(p.name.clone(), eff.features.iter().cloned().collect());
        }
        eff_by_id.insert(p.id.clone(), std::sync::Arc::new(eff.features));
    }

    // Roots from cargo targets; one root per distinct source file.
    let mut roots: Vec<walk::Root> = Vec::new();
    let mut seen_src: HashSet<PathBuf> = HashSet::new();
    for p in &packages {
        for t in &p.targets {
            let cat = if t.kind.iter().any(|k| k == "test" || k == "bench") {
                walk::Cat::Test
            } else if t.kind.iter().any(|k| k == "example" || k == "custom-build") {
                walk::Cat::Tool
            } else {
                walk::Cat::Prod
            };
            if seen_src.insert(t.src_path.clone()) {
                roots.push(walk::Root {
                    path: t.src_path.clone(),
                    cat,
                    package: p.name.clone(),
                    target_name: t.name.clone(),
                    kind: t.kind.join(","),
                    cargo_features: eff_by_id.get(&p.id).cloned().unwrap_or_default(),
                });
            }
        }
    }
    if roots.is_empty() {
        return Err("selected packages have no targets".to_string());
    }

    let base = meta.workspace_root.clone();
    let mut files = walk::load(roots, &base, &target);
    // Drop cfg-false code from every AST before the global and analyze
    // passes count it (file scope labels from the walk stay as they are).
    for f in &mut files.files {
        if let Some(ast) = &mut f.ast {
            cfg_eval::strip_cfg_false(ast, &target.with_features((*f.cargo_features).clone()));
        }
    }

    // External crate roots the path resolver should recognize: the seed
    // list from the inventory plus every dependency name cargo knows.
    // Workspace members are project source, not external crates, even
    // when another member depends on them by path.
    let member_names: HashSet<String> =
        meta.packages.iter().filter(|p| meta.workspace_members.contains(&p.id)).map(|p| p.name.replace('-', "_")).collect();
    let mut ext: HashSet<String> = global::EXT_CRATES.iter().map(|s| s.to_string()).collect();
    for p in &meta.packages {
        for d in &p.dependencies {
            let name = d.name.replace('-', "_");
            if !member_names.contains(&name) {
                ext.insert(name);
                if let Some(r) = &d.rename {
                    ext.insert(r.clone());
                }
            }
        }
    }
    let global = global::collect(&files, &ext);

    let mut stats = stats::Stats::default();
    for (idx, f) in files.files.iter().enumerate() {
        analyze::analyze_file(idx as u32, f, &global, &feats, &mut stats);
    }
    stats.findings.sort_by(|a, b| {
        files.files[a.file_idx as usize]
            .rel
            .cmp(&files.files[b.file_idx as usize].rel)
            .then(a.byte_start.cmp(&b.byte_start))
            .then(a.feature.cmp(&b.feature))
    });
    if let Some(p) = &args.keys_json {
        write_keys_json(p, &stats, &feats)?;
    }

    let deps = report::dep_reports(&packages, &dep_table, &target, &enabled_deps);
    let rep = report::build(
        &manifest.to_string_lossy(),
        &base.to_string_lossy(),
        packages.iter().map(|p| p.name.clone()).collect(),
        cargo_features_report,
        &target,
        t0.elapsed().as_millis(),
        &files,
        &global,
        &stats,
        &feats,
        deps,
        args.summary,
    );
    eprintln!(
        "lust-port: {} files, {} findings, {} parse errors in {:?}",
        rep.totals.files,
        rep.totals.findings,
        rep.totals.parse_errors,
        t0.elapsed()
    );
    if args.json { serde_json::to_string_pretty(&rep).map_err(|e| e.to_string()) } else { Ok(report::render_human(&rep, args.markdown)) }
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    match parse_args(&argv).and_then(run) {
        Ok(out) => println!("{out}"),
        Err(e) if e == USAGE || e.starts_with("usage:") => {
            println!("{e}");
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    }
}
