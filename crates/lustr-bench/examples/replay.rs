//! Recalculate statistics from the public, path-free campaign export.
#![forbid(unsafe_code)]

#[path = "../src/order.rs"]
mod order;
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
    #[serde(default)]
    analysis: Option<Value>,
    #[serde(default)]
    runner_sha256: String,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    lanes: Vec<String>,
    records: Vec<Record>,
    comparisons: Vec<Comparison>,
    #[serde(default)]
    order: Option<Value>,
    #[serde(default)]
    order_method: Option<String>,
    #[serde(default)]
    full_private_report_sha256: String,
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
    #[serde(default)]
    resources: Option<Value>,
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
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal_with_rounding(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(k).is_some_and(|b| equal_with_rounding(v, b)))
        }
        _ => a == b,
    }
}

fn replay(dataset: Dataset) -> Result<Value, String> {
    if dataset.schema != 1 || !(1..=10000).contains(&dataset.pairs) || dataset.cases.is_empty() {
        return Err("schema 1 and nonempty paired dataset required".into());
    }
    let mut results = Vec::new();
    for case in &dataset.cases {
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
            || comparisons
                .iter()
                .any(|[a, b]| a == b || !lanes.contains(a) || !lanes.contains(b))
        {
            return Err(format!(
                "{}: unique lanes and all distinct comparisons required",
                case.name
            ));
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
                || samples
                    .insert((pair, r.lane.as_str()), r.wall_seconds)
                    .is_some()
            {
                return Err(format!("{}: invalid or duplicate timed record", case.name));
            }
        }
        if samples.len() != dataset.pairs * case.lanes.len() {
            return Err(format!("{}: incomplete paired samples", case.name));
        }
        for comparison in &case.comparisons {
            let pairs = (0..dataset.pairs)
                .map(|i| {
                    Ok((
                        *samples
                            .get(&(i, comparison.baseline.as_str()))
                            .ok_or("missing baseline")?,
                        *samples
                            .get(&(i, comparison.candidate.as_str()))
                            .ok_or("missing candidate")?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let summary = stats::summarize(&pairs, dataset.bootstrap_resamples, dataset.seed)?;
            let actual = serde_json::to_value(summary).map_err(|e| e.to_string())?;
            if !equal_with_rounding(&actual, &comparison.analysis) {
                return Err(format!(
                    "{}: published statistics differ from samples",
                    case.name
                ));
            }
            results.push(json!({
                "case": case.name,
                "baseline": comparison.baseline,
                "candidate": comparison.candidate,
                "analysis": actual,
            }));
        }
    }
    let resources = dataset
        .analysis
        .as_ref()
        .map(|v| replay_resources(&dataset, v))
        .transpose()?;
    Ok(
        json!({"verified_comparisons": results.len(), "results": results,"resource_and_aggregate_replay":resources}),
    )
}

fn aggregate(logs: &[Vec<f64>]) -> Result<Value, String> {
    if logs.len() != 3
        || logs
            .iter()
            .any(|v| v.len() != 12 || v.iter().any(|x| !x.is_finite()))
    {
        return Err("three fixed strata of twelve finite paired logs required".into());
    }
    let mean = |v: &[f64]| v.iter().map(|x| x / v.len() as f64).sum::<f64>();
    let point = mean(&logs.iter().map(|v| mean(v)).collect::<Vec<_>>()).exp();
    let mut rng = 6757;
    let mut boot = Vec::with_capacity(10000);
    for _ in 0..10000 {
        let means = logs
            .iter()
            .map(|stratum| {
                let mut sample = Vec::with_capacity(12);
                for _ in 0..12 {
                    let threshold = 12u64.wrapping_neg() % 12;
                    let index = loop {
                        let x = stats::random(&mut rng);
                        if x >= threshold {
                            break (x % 12) as usize;
                        }
                    };
                    sample.push(stratum[index]);
                }
                mean(&sample)
            })
            .collect::<Vec<_>>();
        boot.push(mean(&means).exp());
    }
    boot.sort_by(f64::total_cmp);
    Ok(json!({"direction":"candidate/control", "point":point,
        "conditional_95":[boot[249],boot[9750]], "aggregate_p_value":null}))
}

fn replay_resources(dataset: &Dataset, expected: &Value) -> Result<Value, String> {
    let names = ["t3code-ls", "gitr", "bluefin"];
    let lanes = ["stock-1-99-1", "stage2-control", "stage2-cgu16-thinlto"];
    if dataset.pairs != 12
        || dataset.seed != 6757
        || dataset.bootstrap_resamples != 10000
        || dataset
            .cases
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            != names
    {
        return Err("this resource/aggregate replay requires the fixed CGU16 study".into());
    }
    let analyses = expected["cases"]
        .as_array()
        .ok_or("missing case analyses")?;
    if analyses.len() != 3 {
        return Err("three case analyses required".into());
    }
    let mut logs = Vec::new();
    let mut rss_pass = true;
    let mut max_rss = 0f64;
    for (case, stored) in dataset.cases.iter().zip(analyses) {
        if case.lanes != lanes
            || case.records.len() != 80
            || stored["case"] != case.name
            || case.order_method.as_deref() != Some(order::METHOD)
        {
            return Err(format!(
                "{}: lane, command count, method or case mismatch",
                case.name
            ));
        }
        let mut rng = dataset.seed;
        let warm = order::rounds(3, 1, &mut rng);
        let rounds = order::rounds(3, 12, &mut rng);
        let mut schedule = vec![json!({"pair":0,"warmup":true,"lane_indices":warm[0]})];
        schedule.extend(
            rounds
                .iter()
                .enumerate()
                .map(|(i, row)| json!({"pair":i,"warmup":false,"lane_indices":row})),
        );
        if case.order.as_ref() != Some(&json!(schedule)) {
            return Err("schedule differs from seeded cyclic blocks".into());
        }
        let mut positions = [[0; 3]; 3];
        for row in &rounds {
            for (p, &lane) in row.iter().enumerate() {
                positions[lane][p] += 1;
            }
        }
        if positions != [[4; 3]; 3] {
            return Err("unbalanced timed positions".into());
        }
        for (phase, layout) in [
            ("timed", rounds.as_slice()),
            ("excluded-warmup", warm.as_slice()),
        ] {
            let actual = case
                .records
                .iter()
                .filter(|r| r.phase == phase)
                .map(|r| (r.pair, r.lane.as_str()))
                .collect::<Vec<_>>();
            let intended = layout
                .iter()
                .enumerate()
                .flat_map(|(i, row)| row.iter().map(move |&l| (Some(i), lanes[l])))
                .collect::<Vec<_>>();
            if actual != intended {
                return Err("record sequence differs from planned lane order".into());
            }
        }
        for record in &case.records {
            let packet = record
                .resources
                .as_ref()
                .ok_or("missing resource observation")?;
            let usage = &packet["usage"];
            let expected_exit = if record.phase.ends_with(":invalid-borrow") {
                1
            } else {
                0
            };
            if packet["availability"] != "available"
                || packet["collector_exit"] != 0
                || !packet["collector_signal"].is_null()
                || usage["schema"] != 2
                || usage["collector_sha256"] != dataset.runner_sha256
                || usage["scope"]
                    != "linux-rusage-children-v1: CPU sums waited children; descendants only through fully waited ancestry; peak RSS is largest waited process in KiB, not aggregate tree memory"
                || record.exit != Some(expected_exit)
                || record.signal.is_some()
                || record.timed_out
                || record.output_limit
                || usage["raw_status"].as_i64() != record.exit.map(|x| i64::from(x) * 256)
            {
                return Err("resource status/identity/scope mismatch".into());
            }
            for key in [
                "user_microseconds",
                "system_microseconds",
                "peak_rss_kib",
                "collector_user_microseconds",
                "collector_system_microseconds",
            ] {
                if !usage[key].as_i64().is_some_and(|x| x >= 0) {
                    return Err("invalid integer resource".into());
                }
            }
        }
        let get = |pair, lane| {
            case.records
                .iter()
                .find(|r| r.phase == "timed" && r.pair == Some(pair) && r.lane == lane)
                .ok_or("missing timed row")
        };
        let mut wall = Vec::new();
        let mut cpu = Vec::new();
        let mut rss = Vec::new();
        for pair in 0..12 {
            let control = get(pair, lanes[1])?;
            let candidate = get(pair, lanes[2])?;
            wall.push((candidate.wall_seconds, control.wall_seconds));
            let mut totals = Vec::new();
            let mut peaks = Vec::new();
            for row in [control, candidate] {
                let u = &row.resources.as_ref().ok_or("missing resources")?["usage"];
                let total = u["user_microseconds"]
                    .as_i64()
                    .and_then(|a| {
                        u["system_microseconds"]
                            .as_i64()
                            .and_then(|b| a.checked_add(b))
                    })
                    .filter(|x| *x > 0)
                    .ok_or("missing/nonpositive CPU pair")?;
                totals.push(total as f64);
                peaks.push(
                    u["peak_rss_kib"]
                        .as_i64()
                        .filter(|x| *x > 0)
                        .ok_or("missing/nonpositive RSS pair")?,
                );
            }
            cpu.push((totals[1], totals[0]));
            let passes = 5i128 * i128::from(peaks[1]) <= 6i128 * i128::from(peaks[0]);
            rss_pass &= passes;
            max_rss = max_rss.max(peaks[1] as f64 / peaks[0] as f64);
            rss.push(json!({"pair":pair,"control_peak_kib":peaks[0],"candidate_peak_kib":peaks[1],"passes_exact_5t_le_6c":passes}));
        }
        let primary = stats::summarize(&wall, 10000, 6757)?;
        logs.push(primary.log_ratios.clone());
        let reverse = wall.iter().map(|(a, b)| (*b, *a)).collect::<Vec<_>>();
        for (key, actual) in [
            ("candidate_over_control", serde_json::to_value(&primary)),
            (
                "control_over_candidate",
                serde_json::to_value(stats::summarize(&reverse, 10000, 6757)?),
            ),
            (
                "cpu_candidate_over_control",
                serde_json::to_value(stats::summarize(&cpu, 10000, 6757)?),
            ),
        ] {
            if !equal_with_rounding(&actual.map_err(|e| e.to_string())?, &stored[key]) {
                return Err(format!("{}: {key} differs", case.name));
            }
        }
        if stored["rss_pairs"] != json!(rss)
            || stored["report_sha256"] != case.full_private_report_sha256
        {
            return Err("RSS integer table/report binding differs".into());
        }
    }
    let actual = aggregate(&logs)?;
    let pass = actual["point"].as_f64().ok_or("aggregate point")? <= 0.97
        && actual["conditional_95"][1]
            .as_f64()
            .ok_or("aggregate upper")?
            < 1.0
        && analyses.iter().all(|v| {
            v["candidate_over_control"]["geometric_ratio"]
                .as_f64()
                .is_some_and(|x| x <= 1.03)
        })
        && rss_pass;
    if !equal_with_rounding(&actual, &expected["aggregate"])
        || expected["rss_gate"] != rss_pass
        || !equal_with_rounding(
            &json!(max_rss),
            &expected["maximum_candidate_control_peak_rss_ratio"],
        )
        || expected["numeric_thresholds_pass"] != pass
        || expected["aggregate_ram"] != "UNRUN"
        || expected["energy"] != "UNRUN"
    {
        return Err("aggregate or preregistered numeric decision differs".into());
    }
    Ok(
        json!({"aggregate":actual,"verified_cpu_comparisons":3,"verified_rss_pairs":36,
        "numeric_thresholds_pass":pass,"release_acceptance":false}),
    )
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
