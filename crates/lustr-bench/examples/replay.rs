//! Recalculate statistics from the public, path-free campaign export.
#![forbid(unsafe_code)]

#[path = "../src/stats.rs"]
mod stats;

use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, env, fs};

#[derive(Deserialize)]
struct Dataset {
    schema: u32,
    pairs: usize,
    seed: u64,
    bootstrap_resamples: usize,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    lanes: Vec<String>,
    records: Vec<Record>,
    comparisons: Vec<Comparison>,
}

#[derive(Deserialize)]
struct Record {
    phase: String,
    pair: Option<usize>,
    lane: String,
    wall_seconds: f64,
    exit: Option<i32>,
    signal: Option<i32>,
    timed_out: bool,
    output_limit: bool,
}

#[derive(Deserialize)]
struct Comparison {
    baseline: String,
    candidate: String,
    analysis: Value,
}

fn equal_with_rounding(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            if a.is_u64() && b.is_u64() {
                a == b
            } else {
                let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) else {
                    return false;
                };
                (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(1.0)
            }
        }
        (Value::Array(a), Value::Array(b)) => a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal_with_rounding(a, b)),
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|b| equal_with_rounding(v, b)))
        }
        _ => a == b,
    }
}

fn replay(dataset: Dataset) -> Result<Value, String> {
    if dataset.schema != 1 || !(1..=10000).contains(&dataset.pairs) || dataset.cases.is_empty() {
        return Err("schema 1 and nonempty paired dataset required".into());
    }
    let mut results = Vec::new();
    for case in dataset.cases {
        let lanes: std::collections::BTreeSet<_> = case.lanes.iter().collect();
        let comparisons: std::collections::BTreeSet<_> = case
            .comparisons
            .iter()
            .map(|c| {
                let mut pair = [&c.baseline, &c.candidate];
                pair.sort();
                pair
            })
            .collect();
        if lanes.len() < 2
            || lanes.len() > 8
            || lanes.len() != case.lanes.len()
            || comparisons.len() != lanes.len() * (lanes.len() - 1) / 2
            || comparisons.len() != case.comparisons.len()
            || comparisons.iter().any(|[a, b]| a == b || !lanes.contains(a) || !lanes.contains(b))
        {
            return Err(format!("{}: unique lanes and all distinct comparisons required", case.name));
        }
        let mut samples = BTreeMap::new();
        for r in case.records.iter().filter(|r| r.phase == "timed") {
            let pair = r.pair.ok_or("timed record without pair")?;
            if pair >= dataset.pairs
                || !case.lanes.contains(&r.lane)
                || r.exit != Some(0)
                || r.signal.is_some()
                || r.timed_out
                || r.output_limit
                || !r.wall_seconds.is_finite()
                || r.wall_seconds <= 0.0
                || samples.insert((pair, r.lane.as_str()), r.wall_seconds).is_some()
            {
                return Err(format!("{}: invalid or duplicate timed record", case.name));
            }
        }
        if samples.len() != dataset.pairs * case.lanes.len() {
            return Err(format!("{}: incomplete paired samples", case.name));
        }
        for comparison in case.comparisons {
            let pairs = (0..dataset.pairs)
                .map(|i| {
                    Ok((
                        *samples.get(&(i, comparison.baseline.as_str())).ok_or("missing baseline")?,
                        *samples.get(&(i, comparison.candidate.as_str())).ok_or("missing candidate")?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let summary = stats::summarize(&pairs, dataset.bootstrap_resamples, dataset.seed)?;
            let actual = serde_json::to_value(summary).map_err(|e| e.to_string())?;
            if !equal_with_rounding(&actual, &comparison.analysis) {
                return Err(format!("{}: published statistics differ from samples", case.name));
            }
            results.push(json!({
                "case": case.name,
                "baseline": comparison.baseline,
                "candidate": comparison.candidate,
                "analysis": actual,
            }));
        }
    }
    Ok(json!({"verified_comparisons": results.len(), "results": results}))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let path = args.next().ok_or("usage: replay samples.json")?;
    if args.next().is_some() {
        return Err("usage: replay samples.json".into());
    }
    let data = serde_json::from_slice(&fs::read(path)?)?;
    println!("{}", serde_json::to_string_pretty(&replay(data)?)?);
    Ok(())
}
