//! Builds the report value and renders it as JSON, plain text, or
//! markdown. Human output is grouped by root cause and by feature: it
//! never prints one line per finding.

use crate::cfg_eval::{TargetInfo, Tri, eval_cfg_str};
use crate::features::FeatureTable;
use crate::global::Global;
use crate::stats::{Finding, Stats};
use crate::walk::{Cat, Files};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;

#[derive(Serialize, Clone)]
pub struct FindingOut {
    pub feature: String,
    pub raw_key: String,
    pub file: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
    pub scope: String,
    pub fix_class: String,
    pub release_safe_rewrite: bool,
}

#[derive(Serialize)]
pub struct FeatureSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub milestone: Option<i64>,
    pub milestone_label: String,
    pub inventory_step: Option<String>,
    pub root_cause: String,
    pub fix_class: String,
    pub release_safe_rewrite: bool,
    pub release_impact: String,
    pub notes: String,
    pub findings: usize,
    /// Raw primary-key uses across all categories (inventory style count).
    pub raw_uses: u64,
    pub prod_uses: u64,
    pub files: usize,
    pub prod_files: usize,
    pub prod_fns: usize,
    pub scopes: BTreeMap<String, usize>,
    pub samples: Vec<FindingOut>,
}

#[derive(Serialize)]
pub struct Group {
    pub root_cause: String,
    pub findings: usize,
    pub files: usize,
    pub features: Vec<String>,
}

#[derive(Serialize)]
pub struct FileReport {
    pub file: String,
    pub lines: usize,
    pub cat: String,
    pub scope: String,
    pub generated: bool,
    pub parse_error: bool,
    pub missing_features: Vec<String>,
    pub findings: usize,
}

#[derive(Serialize, Clone)]
pub struct CurveStep {
    pub id: String,
    pub label: String,
    pub files: usize,
    pub lines: usize,
    pub files_pct: f64,
    pub lines_pct: f64,
}

#[derive(Serialize)]
pub struct Coverage {
    pub universe: String,
    pub total_files: usize,
    pub total_lines: usize,
    pub supported_today_files: usize,
    pub by_plan_milestone: Vec<CurveStep>,
    pub by_inventory_step: Vec<CurveStep>,
    pub files_per_step_use: Vec<CurveStep>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DepDecision {
    pub name: String,
    pub decision: String,
    pub milestone: Option<i64>,
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // schema_version guards future table formats
pub struct DepTable {
    pub schema_version: u32,
    pub dependencies: Vec<DepDecision>,
}

pub const EMBEDDED_DEPS: &str = include_str!("../dependencies.json");

impl DepTable {
    pub fn embedded() -> Result<DepTable, String> {
        serde_json::from_str(EMBEDDED_DEPS).map_err(|e| format!("bad dependencies.json: {e}"))
    }
    pub fn lookup(&self, name: &str) -> Option<&DepDecision> {
        let alt1 = name.replace('-', "_");
        let alt2 = name.replace('_', "-");
        self.dependencies.iter().find(|d| d.name == name || d.name == alt1 || d.name == alt2)
    }
}

#[derive(Serialize, Clone)]
pub struct DepReport {
    pub name: String,
    pub req: String,
    pub kind: String,
    pub target: Option<String>,
    pub applies_to_target: String,
    pub optional: bool,
    /// True when a non-optional dep, or an optional dep some using
    /// package enables under its effective dev features.
    pub enabled: bool,
    pub features: Vec<String>,
    pub used_by: Vec<String>,
    pub decision: String,
    pub milestone: Option<i64>,
    pub notes: String,
}

#[derive(Serialize)]
pub struct Totals {
    pub files: usize,
    pub lines: usize,
    pub fns: usize,
    pub files_by_cat: BTreeMap<String, usize>,
    pub lines_by_cat: BTreeMap<String, usize>,
    pub inline_test_lines_in_prod: usize,
    pub findings: usize,
    pub files_with_findings: usize,
    pub parse_errors: usize,
    pub missing_modules: usize,
    pub undecidable_cfg: usize,
}

#[derive(Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub tool: String,
    pub manifest_path: String,
    pub workspace_root: String,
    pub packages: Vec<String>,
    /// Effective Cargo features per scanned package (dev config applied).
    pub cargo_features: BTreeMap<String, Vec<String>>,
    pub target: String,
    pub elapsed_ms: u128,
    pub totals: Totals,
    pub root_causes: Vec<Group>,
    pub features: Vec<FeatureSummary>,
    pub dependencies: Vec<DepReport>,
    pub coverage: Coverage,
    pub coverage_all_prod: Coverage,
    pub files: Vec<FileReport>,
    pub findings: Vec<FindingOut>,
    pub findings_total: usize,
    pub findings_truncated: bool,
    pub parse_errors: Vec<BTreeMap<String, String>>,
    pub missing_modules: Vec<BTreeMap<String, String>>,
    pub undecidable_cfg: Vec<String>,
    /// Aggregate raw-key counts by category, top level groups only in
    /// summary mode. Full raw counts stay in stats for feature summaries.
    pub lifetime_type_count: usize,
    pub macro_def_count: usize,
    pub macro_name_count: usize,
}

fn finding_out(f: &Finding, files: &Files, feats: &FeatureTable) -> FindingOut {
    let feat = feats.get(&f.feature);
    FindingOut {
        feature: f.feature.clone(),
        raw_key: f.raw_key.clone(),
        file: files.files[f.file_idx as usize].rel.clone(),
        byte_start: f.byte_start,
        byte_end: f.byte_end,
        line: f.line,
        column: f.column,
        snippet: f.snippet.clone(),
        scope: f.scope.to_string(),
        fix_class: feat.map(|x| x.fix_label().to_string()).unwrap_or_else(|| "manual".into()),
        release_safe_rewrite: feat.map(|x| x.release_safe_rewrite).unwrap_or(false),
    }
}

const INV_ORDER: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7a", "7b", "8", "9", "10"];
const INV_NAMES: &[(&str, &str)] = &[
    ("0", "codemods (macros, impl-Trait args, async, #[path])"),
    ("1", "M4 core syntax and std"),
    ("2", "attributes and cfg"),
    ("3", "traits and generics"),
    ("4", "closures and iterators"),
    ("5", "std runtime (Rc, RefCell, Fx/IndexMap, time)"),
    ("6", "dyn Trait and Any"),
    ("7a", "'static, '_ and std lifetime types"),
    ("7b", "lifetime params on user types, named lifetimes"),
    ("8", "threads and sync"),
    ("9", "panic unwinding"),
    ("10", "C interop and system crates"),
];

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 { 0.0 } else { (n as f64 * 1000.0 / d as f64).round() / 10.0 }
}

fn curve(universe: &[(usize, usize, BTreeSet<String>)], feats: &FeatureTable, label: &str) -> Coverage {
    // universe entries: (file idx, lines, missing feature ids)
    let total_files = universe.len();
    let total_lines: usize = universe.iter().map(|u| u.1).sum();
    let supported_today = universe.iter().filter(|u| u.2.is_empty()).count();
    let mut by_m: Vec<CurveStep> = Vec::new();
    for n in [0i64, 4, 5, 6, 7, 8] {
        let ok: Vec<_> =
            universe.iter().filter(|u| u.2.iter().all(|id| matches!(feats.get(id).and_then(|f| f.milestone), Some(m) if m <= n))).collect();
        let lines: usize = ok.iter().map(|u| u.1).sum();
        by_m.push(CurveStep {
            id: n.to_string(),
            label: if n == 0 { "supported today".into() } else { feats.milestone_label(Some(n)) },
            files: ok.len(),
            lines,
            files_pct: pct(ok.len(), total_files),
            lines_pct: pct(lines, total_lines),
        });
    }
    let mut by_inv: Vec<CurveStep> = Vec::new();
    let mut files_per_step: Vec<CurveStep> = Vec::new();
    let mut done: HashSet<&str> = HashSet::new();
    for step in INV_ORDER {
        done.insert(step);
        let ok: Vec<_> = universe
            .iter()
            .filter(|u| {
                u.2.iter().all(|id| match feats.get(id).and_then(|f| f.inventory_step.as_deref()) {
                    Some(s) => done.contains(s),
                    None => false,
                })
            })
            .collect();
        let lines: usize = ok.iter().map(|u| u.1).sum();
        let name = INV_NAMES.iter().find(|(k, _)| k == step).map(|(_, v)| *v).unwrap_or("");
        by_inv.push(CurveStep {
            id: step.to_string(),
            label: format!("{step}. {name}"),
            files: ok.len(),
            lines,
            files_pct: pct(ok.len(), total_files),
            lines_pct: pct(lines, total_lines),
        });
        let users: Vec<_> = universe
            .iter()
            .filter(|u| u.2.iter().any(|id| feats.get(id).and_then(|f| f.inventory_step.as_deref()) == Some(step)))
            .collect();
        let ulines: usize = users.iter().map(|u| u.1).sum();
        files_per_step.push(CurveStep {
            id: step.to_string(),
            label: format!("{step}. {name}"),
            files: users.len(),
            lines: ulines,
            files_pct: pct(users.len(), total_files),
            lines_pct: pct(ulines, total_lines),
        });
    }
    Coverage {
        universe: label.to_string(),
        total_files,
        total_lines,
        supported_today_files: supported_today,
        by_plan_milestone: by_m,
        by_inventory_step: by_inv,
        files_per_step_use: files_per_step,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build(
    manifest_path: &str,
    workspace_root: &str,
    packages: Vec<String>,
    cargo_features: BTreeMap<String, Vec<String>>,
    target: &TargetInfo,
    elapsed_ms: u128,
    files: &Files,
    global: &Global,
    stats: &Stats,
    feats: &FeatureTable,
    deps: Vec<DepReport>,
    summary: bool,
) -> Report {
    // Per-file raw keys. Use the bucket of the file's own category, so a
    // test-only hit inside a prod file does not block the prod build.
    let mut per_file_keys: Vec<HashSet<String>> = vec![HashSet::new(); files.files.len()];
    for (key, e) in &stats.map {
        for c in 0..5 {
            for &fi in &e.files[c] {
                if files.files[fi as usize].cat as usize == c {
                    per_file_keys[fi as usize].insert(key.clone());
                }
            }
        }
    }
    let mut findings_by_file: HashMap<u32, usize> = HashMap::new();
    for f in &stats.findings {
        *findings_by_file.entry(f.file_idx).or_default() += 1;
    }

    let mut file_reports: Vec<FileReport> = Vec::new();
    let mut missing_by_file: Vec<BTreeSet<String>> = Vec::new();
    for (i, file) in files.files.iter().enumerate() {
        let mut missing: BTreeSet<String> = BTreeSet::new();
        for k in &per_file_keys[i] {
            if let Some(feat) = feats.classify(k) {
                if feat.is_gap() {
                    missing.insert(feat.id.clone());
                }
            }
        }
        // Findings may come from buckets of another category (shared file).
        // They still count for the file report, but not for coverage above.
        missing_by_file.push(missing.clone());
        file_reports.push(FileReport {
            file: file.rel.clone(),
            lines: file.lines,
            cat: file.cat.name().to_string(),
            scope: file.scope().to_string(),
            generated: file.generator.is_some(),
            parse_error: file.parse_error.is_some(),
            missing_features: missing.into_iter().collect(),
            findings: findings_by_file.get(&(i as u32)).copied().unwrap_or(0),
        });
    }

    // Feature summaries.
    let mut feat_summaries: Vec<FeatureSummary> = Vec::new();
    for feat in feats.features.iter().filter(|f| f.is_gap()) {
        let hits: Vec<&Finding> = stats.findings.iter().filter(|f| f.feature == feat.id).collect();
        let mut scopes: BTreeMap<String, usize> = BTreeMap::new();
        for h in &hits {
            *scopes.entry(h.scope.to_string()).or_default() += 1;
        }
        let hit_files: HashSet<u32> = hits.iter().map(|h| h.file_idx).collect();
        // Raw primary-key counts in inventory style.
        let (mut raw, mut prod) = (0u64, 0u64);
        let mut prod_files: HashSet<u32> = HashSet::new();
        let mut prod_fns: HashSet<u32> = HashSet::new();
        for (key, e) in &stats.map {
            if FeatureTable::is_primary(feat, key) && feats.classify(key).map(|f| f.id.as_str()) == Some(feat.id.as_str()) {
                raw += e.count.iter().sum::<u64>();
                prod += e.count[Cat::Prod as usize] + e.count[Cat::Gen as usize];
                prod_files.extend(e.files[Cat::Prod as usize].iter().copied());
                prod_fns.extend(e.fns[Cat::Prod as usize].iter().copied());
            }
        }
        // Files that need this feature include files with only non-primary keys.
        let need_files: HashSet<usize> = missing_by_file.iter().enumerate().filter(|(_, m)| m.contains(&feat.id)).map(|(i, _)| i).collect();
        let mut all_files: HashSet<usize> = hit_files.iter().map(|&i| i as usize).collect();
        all_files.extend(need_files.iter().copied());
        let samples: Vec<FindingOut> = hits.iter().take(3).map(|f| finding_out(f, files, feats)).collect();
        feat_summaries.push(FeatureSummary {
            id: feat.id.clone(),
            title: feat.title.clone(),
            status: feat.status.clone(),
            milestone: feat.milestone,
            milestone_label: feats.milestone_label(feat.milestone),
            inventory_step: feat.inventory_step.clone(),
            root_cause: feat.root_cause.clone(),
            fix_class: feat.fix_label().to_string(),
            release_safe_rewrite: feat.release_safe_rewrite,
            release_impact: feat.release_impact.clone(),
            notes: feat.notes.clone(),
            findings: hits.len(),
            raw_uses: raw,
            prod_uses: prod,
            files: all_files.len(),
            prod_files: prod_files.len(),
            prod_fns: prod_fns.len(),
            scopes,
            samples,
        });
    }
    feat_summaries.sort_by(|a, b| b.findings.cmp(&a.findings).then(a.id.cmp(&b.id)));

    // Root-cause groups.
    let mut group_map: BTreeMap<String, (usize, HashSet<usize>, Vec<String>)> = BTreeMap::new();
    for fs in &feat_summaries {
        if fs.findings == 0 && fs.files == 0 {
            continue;
        }
        let e = group_map.entry(fs.root_cause.clone()).or_default();
        e.0 += fs.findings;
        for (i, m) in missing_by_file.iter().enumerate() {
            if m.contains(&fs.id) {
                e.1.insert(i);
            }
        }
        e.2.push(fs.id.clone());
    }
    let mut groups: Vec<Group> =
        group_map.into_iter().map(|(k, (n, fl, ids))| Group { root_cause: k, findings: n, files: fl.len(), features: ids }).collect();
    groups.sort_by(|a, b| b.findings.cmp(&a.findings));

    // Coverage universes: active prod (cfg not false) and all prod.
    let mk = |pred: &dyn Fn(&crate::walk::File) -> bool| -> Vec<(usize, usize, BTreeSet<String>)> {
        files
            .files
            .iter()
            .enumerate()
            .filter(|(_, f)| pred(f))
            .map(|(i, f)| {
                let mut m = missing_by_file[i].clone();
                if f.parse_error.is_some() {
                    m.insert("parse.error".to_string());
                }
                (i, f.lines, m)
            })
            .collect()
    };
    let active = mk(&|f| f.cat == Cat::Prod && f.cfg_state != Tri::False);
    let all_prod = mk(&|f| f.cat == Cat::Prod);
    let coverage = curve(&active, feats, "prod files active on this target (lib and bin roots, generated files excluded)");
    let coverage_all = curve(&all_prod, feats, "all prod files, every target (inventory-comparable)");

    let mut findings_out: Vec<FindingOut> = Vec::new();
    let truncated;
    if summary {
        let mut per_feat: HashMap<String, usize> = HashMap::new();
        for f in &stats.findings {
            let c = per_feat.entry(f.feature.clone()).or_default();
            if *c < 5 {
                findings_out.push(finding_out(f, files, feats));
                *c += 1;
            }
        }
        truncated = findings_out.len() < stats.findings.len();
    } else {
        findings_out = stats.findings.iter().map(|f| finding_out(f, files, feats)).collect();
        truncated = false;
    }
    findings_out.sort_by(|a, b| a.file.cmp(&b.file).then(a.byte_start.cmp(&b.byte_start)).then(a.feature.cmp(&b.feature)));

    let mut files_by_cat: BTreeMap<String, usize> = BTreeMap::new();
    let mut lines_by_cat: BTreeMap<String, usize> = BTreeMap::new();
    for c in crate::walk::CATS {
        files_by_cat.insert(c.name().into(), stats.files_by_cat[c as usize]);
        lines_by_cat.insert(c.name().into(), stats.lines_by_cat[c as usize]);
    }
    let files_with_findings = findings_by_file.len();

    Report {
        schema_version: 1,
        tool: "lust-port scan".into(),
        manifest_path: manifest_path.to_string(),
        workspace_root: workspace_root.to_string(),
        packages,
        cargo_features,
        target: target.triple.clone(),
        elapsed_ms,
        totals: Totals {
            files: files.files.len(),
            lines: files.files.iter().map(|f| f.lines).sum(),
            fns: stats.fns.len(),
            files_by_cat,
            lines_by_cat,
            inline_test_lines_in_prod: stats.inline_test_lines[Cat::Prod as usize],
            findings: stats.findings.len(),
            files_with_findings,
            parse_errors: files.parse_errors.len(),
            missing_modules: files.missing_mods.len(),
            undecidable_cfg: files.undecidable_cfg.len(),
        },
        root_causes: groups,
        features: feat_summaries,
        dependencies: deps,
        coverage,
        coverage_all_prod: coverage_all,
        files: file_reports,
        findings_total: stats.findings.len(),
        findings: findings_out,
        findings_truncated: truncated,
        parse_errors: files
            .parse_errors
            .iter()
            .map(|(f, e)| BTreeMap::from([("file".to_string(), f.clone()), ("error".to_string(), e.clone())]))
            .collect(),
        missing_modules: files
            .missing_mods
            .iter()
            .map(|m| {
                BTreeMap::from([
                    ("file".to_string(), m.from_file.clone()),
                    ("module".to_string(), m.module.clone()),
                    ("tried".to_string(), m.tried.join(", ")),
                ])
            })
            .collect(),
        undecidable_cfg: files.undecidable_cfg.clone(),
        lifetime_type_count: global.lt_types.len(),
        macro_def_count: global.macro_defs.len(),
        macro_name_count: global.macro_names.len(),
    }
}

/// Aggregates direct dependencies of the scanned packages.
/// `enabled_deps` maps a package name to the optional dependency names
/// its effective dev features enable.
pub fn dep_reports(
    packages: &[&crate::cargo_meta::Package],
    table: &DepTable,
    target: &TargetInfo,
    enabled_deps: &HashMap<String, HashSet<String>>,
) -> Vec<DepReport> {
    let mut map: BTreeMap<(String, String, String), DepReport> = BTreeMap::new();
    for p in packages {
        for d in &p.dependencies {
            let kind = d.kind.clone().unwrap_or_else(|| "normal".into());
            let tgt = d.target.clone().unwrap_or_default();
            let enabled_here = !d.optional || enabled_deps.get(&p.name).is_some_and(|s| s.contains(&d.name));
            let e = map.entry((d.name.clone(), kind.clone(), tgt)).or_insert_with(|| {
                let (decision, milestone, notes) = match table.lookup(&d.name) {
                    Some(x) => (x.decision.clone(), x.milestone, x.notes.clone()),
                    None => ("unknown".into(), None, "No decision yet. Add one to dependencies.json after review.".into()),
                };
                let applies = match &d.target {
                    None => "yes".to_string(),
                    Some(t) => eval_cfg_str(t, target).name().to_string(),
                };
                DepReport {
                    name: d.rename.clone().unwrap_or_else(|| d.name.clone()),
                    req: d.req.clone(),
                    kind,
                    target: d.target.clone(),
                    applies_to_target: applies,
                    optional: d.optional,
                    enabled: enabled_here,
                    features: d.features.clone(),
                    used_by: Vec::new(),
                    decision,
                    milestone,
                    notes,
                }
            });
            e.enabled |= enabled_here;
            if !e.used_by.contains(&p.name) {
                e.used_by.push(p.name.clone());
            }
        }
    }
    map.into_values().collect()
}

fn num(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn curve_rows(cov: &Coverage, md: bool) -> String {
    let mut o = String::new();
    if md {
        let _ = writeln!(o, "| after plan milestone | files | files % | lines % |");
        let _ = writeln!(o, "|---|---|---|---|");
        for s in &cov.by_plan_milestone {
            let label = if s.id == "0" { "supported today".to_string() } else { format!("milestone {}", s.id) };
            let _ = writeln!(o, "| {} | {} | {} | {} |", label, num(s.files), s.files_pct, s.lines_pct);
        }
        let _ = writeln!(o, "\n| after inventory step | files | files % | lines % |");
        let _ = writeln!(o, "|---|---|---|---|");
        for s in &cov.by_inventory_step {
            let _ = writeln!(o, "| {} | {} | {} | {} |", s.label, num(s.files), s.files_pct, s.lines_pct);
        }
        let _ = writeln!(o, "\n| files that use each inventory step | files | lines |");
        let _ = writeln!(o, "|---|---|---|");
        for s in &cov.files_per_step_use {
            let _ = writeln!(o, "| {} | {} | {} |", s.label, num(s.files), num(s.lines));
        }
    } else {
        let _ = writeln!(o, "  {:<18}{:>8}{:>9}{:>9}", "after milestone", "files", "files %", "lines %");
        for s in &cov.by_plan_milestone {
            let label = if s.id == "0" { "today".to_string() } else { format!("milestone {}", s.id) };
            let _ = writeln!(o, "  {:<18}{:>8}{:>9.1}{:>9.1}", label, s.files, s.files_pct, s.lines_pct);
        }
        let _ = writeln!(o, "  {:<58}{:>8}{:>9}{:>9}", "after inventory step", "files", "files %", "lines %");
        for s in &cov.by_inventory_step {
            let _ = writeln!(o, "  {:<58}{:>8}{:>9.1}{:>9.1}", s.label, s.files, s.files_pct, s.lines_pct);
        }
    }
    o
}

pub fn render_human(r: &Report, md: bool) -> String {
    let mut o = String::new();
    let h = |o: &mut String, level: usize, text: &str| {
        if md {
            let _ = writeln!(o, "\n{} {}\n", "#".repeat(level), text);
        } else {
            let _ = writeln!(o, "\n{text}\n{}", "-".repeat(text.chars().count().min(60)));
        }
    };
    if md {
        let _ = writeln!(o, "# lust port scan: {}\n", r.packages.join(", "));
        let _ = writeln!(o, "Generated by `lust-port scan --markdown`. Do not edit the numbers by hand; run the tool again.\n");
    } else {
        let _ = writeln!(o, "lust port scan: {}", r.packages.join(", "));
    }
    let _ = writeln!(o, "manifest: {}", r.manifest_path);
    let _ = writeln!(o, "target: {}", r.target);
    let _ = writeln!(o, "scan time: {} ms", r.elapsed_ms);
    let t = &r.totals;
    let _ = writeln!(
        o,
        "files: {} ({} lines, {} functions), findings: {} in {} files, parse errors: {}, missing modules: {}, undecidable cfg: {}",
        num(t.files),
        num(t.lines),
        num(t.fns),
        num(t.findings),
        num(t.files_with_findings),
        t.parse_errors,
        t.missing_modules,
        t.undecidable_cfg
    );
    if md {
        let _ = writeln!(o, "\n| category | files | lines |\n|---|---|---|");
        for (k, v) in &t.files_by_cat {
            let _ = writeln!(o, "| {} | {} | {} |", k, num(*v), num(*t.lines_by_cat.get(k).unwrap_or(&0)));
        }
    } else {
        for (k, v) in &t.files_by_cat {
            let _ = writeln!(o, "  {:<8}{:>6} files {:>9} lines", k, v, num(*t.lines_by_cat.get(k).unwrap_or(&0)));
        }
    }
    let _ = writeln!(
        o,
        "lifetime-carrying user types: {}, macro_rules definitions: {} ({} names)",
        r.lifetime_type_count, r.macro_def_count, r.macro_name_count
    );

    h(&mut o, 2, "Gaps by root cause");
    if md {
        let _ = writeln!(o, "| root cause | findings | files | features |\n|---|---|---|---|");
        for g in &r.root_causes {
            let _ = writeln!(o, "| {} | {} | {} | {} |", g.root_cause, num(g.findings), num(g.files), g.features.join(", "));
        }
    } else {
        for g in &r.root_causes {
            let _ = writeln!(o, "  {:<22}{:>8} findings {:>6} files   {}", g.root_cause, num(g.findings), g.files, g.features.join(", "));
        }
    }

    h(&mut o, 2, "Gaps by feature");
    let _ = writeln!(
        o,
        "A finding is one primary-key hit. `prod uses` is the same count style as INVENTORY.md (prod plus generated files, before dedup by span). At most 2 samples per feature are shown here; use --json for every finding.\n"
    );
    if md {
        let _ = writeln!(o, "| feature | milestone | inventory step | fix | release-safe rewrite | findings | prod uses | files |");
        let _ = writeln!(o, "|---|---|---|---|---|---|---|---|");
    }
    for f in &r.features {
        if f.findings == 0 && f.files == 0 {
            continue;
        }
        let ms = f.milestone.map(|m| m.to_string()).unwrap_or_else(|| "none".into());
        let inv = f.inventory_step.clone().unwrap_or_else(|| "-".into());
        let yn = if f.release_safe_rewrite { "yes" } else { "no" };
        if md {
            let _ = writeln!(
                o,
                "| `{}` {} | {} | {} | {} | {} | {} | {} | {} |",
                f.id,
                f.title.replace('|', "\\|"),
                ms,
                inv,
                f.fix_class,
                yn,
                num(f.findings),
                num(f.prod_uses as usize),
                num(f.files)
            );
        } else {
            let _ = writeln!(o, "{} (milestone {}, step {}, {}, release-safe rewrite: {})", f.id, ms, inv, f.fix_class, yn);
            let _ = writeln!(o, "  {}", f.title);
            let scopes: Vec<String> = f.scopes.iter().map(|(k, v)| format!("{k} {v}")).collect();
            let _ = writeln!(
                o,
                "  findings {} ({}), prod uses {}, files {}, prod files {}, prod fns {}",
                num(f.findings),
                scopes.join(", "),
                num(f.prod_uses as usize),
                f.files,
                f.prod_files,
                f.prod_fns
            );
            for s in f.samples.iter().take(2) {
                let _ = writeln!(o, "    {}:{} [{}..{}] {}", s.file, s.line, s.byte_start, s.byte_end, s.snippet);
            }
        }
    }
    if md {
        let _ = writeln!(o, "\nSamples (2 per feature):\n");
        for f in &r.features {
            if f.samples.is_empty() {
                continue;
            }
            for s in f.samples.iter().take(2) {
                let _ = writeln!(
                    o,
                    "- `{}` {}:{} bytes {}..{} ({}): `{}`",
                    f.id,
                    s.file,
                    s.line,
                    s.byte_start,
                    s.byte_end,
                    s.scope,
                    s.snippet.replace('`', "'")
                );
            }
        }
    }

    h(&mut o, 2, "Coverage: files that would compile");
    let _ = writeln!(
        o,
        "Universe: {} ({} files, {} lines). Supported today with no milestone: {} files.\n",
        r.coverage.universe,
        num(r.coverage.total_files),
        num(r.coverage.total_lines),
        r.coverage.supported_today_files
    );
    o.push_str(&curve_rows(&r.coverage, md));
    let _ = writeln!(
        o,
        "\nInventory-comparable universe: {} ({} files, {} lines). Supported today: {} files.\n",
        r.coverage_all_prod.universe,
        num(r.coverage_all_prod.total_files),
        num(r.coverage_all_prod.total_lines),
        r.coverage_all_prod.supported_today_files
    );
    o.push_str(&curve_rows(&r.coverage_all_prod, md));

    h(&mut o, 2, "Dependencies");
    let _ = writeln!(o, "Release always uses the original crates through Cargo. The decision below is only the dev path.\n");
    if md {
        let _ = writeln!(
            o,
            "| crate | req | kind | target cfg | on this target | used by | decision | milestone |\n|---|---|---|---|---|---|---|---|"
        );
        for d in &r.dependencies {
            let _ = writeln!(
                o,
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                d.name,
                d.req,
                d.kind,
                d.target.clone().unwrap_or_else(|| "-".into()),
                d.applies_to_target,
                d.used_by.join(", "),
                d.decision,
                d.milestone.map(|m| m.to_string()).unwrap_or_else(|| "-".into())
            );
        }
    } else {
        for d in &r.dependencies {
            let _ = writeln!(o, "  {:<28}{:<22} {} ({}) used by {}", d.name, d.decision, d.req, d.kind, d.used_by.join(", "));
        }
    }

    h(&mut o, 2, "Problems found while scanning");
    if r.parse_errors.is_empty() && r.missing_modules.is_empty() && r.undecidable_cfg.is_empty() {
        let _ = writeln!(o, "None.");
    }
    for e in r.parse_errors.iter().take(20) {
        let _ = writeln!(
            o,
            "- parse error {}: {}",
            e.get("file").map(|s| s.as_str()).unwrap_or(""),
            e.get("error").map(|s| s.as_str()).unwrap_or("")
        );
    }
    if r.parse_errors.len() > 20 {
        let _ = writeln!(o, "- and {} more parse errors (see --json)", r.parse_errors.len() - 20);
    }
    for e in r.missing_modules.iter().take(20) {
        let _ = writeln!(
            o,
            "- missing module `{}` from {} (tried: {})",
            e.get("module").map(|s| s.as_str()).unwrap_or(""),
            e.get("file").map(|s| s.as_str()).unwrap_or(""),
            e.get("tried").map(|s| s.as_str()).unwrap_or("")
        );
    }
    if r.missing_modules.len() > 20 {
        let _ = writeln!(o, "- and {} more missing modules (see --json)", r.missing_modules.len() - 20);
    }
    for c in r.undecidable_cfg.iter().take(20) {
        let _ = writeln!(o, "- undecidable {c}");
    }
    if r.undecidable_cfg.len() > 20 {
        let _ = writeln!(o, "- and {} more undecidable cfg entries (see --json)", r.undecidable_cfg.len() - 20);
    }
    let _ = writeln!(
        o,
        "\nLimits: the scanner has no type information. Method names are not tied to receivers, paths resolve through use items and crate preludes, and macro bodies are not expanded. An unmatched construct is treated as supported, so new std or crate paths need a rule in features.json."
    );
    o
}
