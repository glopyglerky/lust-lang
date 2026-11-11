#!/usr/bin/env python3
"""Render native qualification evidence without changing its acceptance decision."""
from __future__ import annotations

import argparse
import gzip
import html
import importlib.util
import json
import math
import os
from pathlib import Path
import zlib


REQUIREMENTS = ("identities", "provenance", "input-stability", "upstream-preservation",
                "upstream-accepted-corpus", "engine-trace", "concurrent-cargo", "mixed-compatibility",
                "full-rust-compatibility", "build-scripts", "doctests")
WORKFLOW_CHECKS = ("source_before", "invocations", "acceptance_tests", "native_build", "artifacts",
                   "protocol", "unsafe", "mixed_generics", "vendor", "safe_library", "package",
                   "relocation", "upstream_coverage", "native_speed", "source_after")


def helper(filename: str):
    path = Path(__file__).with_name(filename)
    spec = importlib.util.spec_from_file_location(filename.replace("-", "_"), path)
    if spec is None or spec.loader is None:
        raise ValueError(f"cannot load {filename}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def expected_series() -> list[str]:
    return helper("qualify-native-speed.py").expected_series()


def cell(value) -> str:
    text = " ".join(str(value).split())
    text = html.escape(text, quote=False).replace("\\", "\\\\")
    for character in "|`[]*":
        text = text.replace(character, "\\" + character)
    return text or "unavailable"


def number(value, scale: float = 1.0) -> str:
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        return "unavailable"
    return f"{value / scale:.3f}"


def load_record(path: Path) -> dict:
    data = gzip.decompress(path.read_bytes()) if path.suffix == ".gz" else path.read_bytes()
    record = json.loads(data)
    if not isinstance(record, dict):
        raise ValueError("report must be a JSON object")
    return record


def render(report: dict | None, receipt: dict | None = None, steps: dict | None = None,
           source_commit: str = "", errors: list[str] | None = None) -> str:
    report = report or {}
    if receipt is None:
        receipt = report.get("native_build_receipt", {})
    errors = errors or []
    required = expected_series()
    series = report.get("series", {})
    requirements = report.get("requirements", {})
    decision = report.get("decision", {})
    source = report.get("source", {})
    build = receipt.get("build", {})
    build_contract = helper("build-native-cargo.py")
    annotations = []
    if (receipt.get("schema") != build_contract.SCHEMA or build.get("profile") != "release"
            or build.get("settings") != build_contract.SETTINGS):
        annotations.append("Native receipt does not describe the required release build recipe.")
    measured_commit = source.get("commit")
    if not measured_commit or receipt.get("source", {}).get("commit") != measured_commit:
        annotations.append("Measured source and native receipt source are absent or different.")
    if source_commit and source_commit != measured_commit:
        annotations.append("Requested source differs from the measured source.")
    for name in WORKFLOW_CHECKS:
        if (steps or {}).get(name, {}).get("outcome") != "success":
            annotations.append(f"Workflow check {name} did not report success.")
    for name, step in (steps or {}).items():
        if name not in WORKFLOW_CHECKS and step.get("outcome") in ("failure", "cancelled"):
            annotations.append(f"Workflow step {name} reported {step['outcome']}.")
    complete = (all(series.get(name, {}).get("state") == "passed" for name in required)
                and all(requirements.get(name, {}).get("state") == "passed" for name in REQUIREMENTS)
                and all(row.get("state") == "passed" for row in requirements.values()))
    qualified = (not errors and not annotations and complete and report.get("qualified") is True
                 and decision.get("qualified") is True and not report.get("failures")
                 and not report.get("diagnostic_without_trace") and not report.get("functional_only"))
    lines = ["# Native compiler qualification", "",
             "**QUALIFIED for the recorded corpus and environment.**" if qualified else
             "**UNQUALIFIED. Failed, missing or inconclusive evidence blocks the installable archive.**",
             "", "Cold and warm speed floors both apply. No faster mode offsets a slower mode.",
             "Hosted-runner timings include environmental noise. This report proves no universal speed or language-coverage claim.", ""]
    if errors:
        lines += ["## Evidence collection errors", ""]
        lines += ["- " + cell(error) for error in errors]
        lines.append("")

    if annotations:
        lines += ["## Summary consistency checks", ""]
        lines += ["- " + cell(annotation) for annotation in annotations]
        lines += ["", "These checks annotate the evidence. They do not inspect the executable or replace the release admission gate.", ""]
    lines += ["## Source and compiler builds", "", "| Field | Value |", "| --- | --- |",
              f"| Requested source | {cell(source_commit)} |",
              f"| Measured source | {cell(source.get('commit', 'unavailable'))} |",
              f"| Source tree | {cell(source.get('tree', 'unavailable'))} |",
              f"| Evidence schema | {cell(report.get('schema', 'unavailable'))} |",
              f"| Native receipt schema | {cell(receipt.get('schema', 'unavailable'))} |",
              f"| Native profile | {cell(build.get('profile', 'unavailable'))} |",
              f"| Native build settings | {cell(json.dumps(build.get('settings', {}), sort_keys=True))} |",
              f"| Runner | {cell(json.dumps(report.get('host', {}), sort_keys=True))} |", ""]
    if build.get("profile") == "gate":
        lines += ["Historical gate-profile build. Its rows remain visible, but they are not release-profile performance qualification.", ""]
    lines += ["| Executable | Version | SHA-256 |", "| --- | --- | --- |"]
    for name, tool in report.get("tools", {}).items():
        version = tool.get("version", tool.get("verbose_version", "unavailable")).splitlines()
        lines.append(f"| {cell(name)} | {cell(version[0] if version else 'unavailable')} | {cell(tool.get('sha256', 'unavailable'))} |")
    lines.append("")

    if steps:
        lines += ["## Workflow steps", "", "| Step | Outcome |", "| --- | --- |"]
        for name, step in steps.items():
            lines.append(f"| {cell(name)} | {cell(step.get('outcome', step.get('conclusion', 'unavailable')))} |")
        lines.append("")

    lines += ["## Compatibility and evidence requirements", "", "| Requirement | State | Detail |", "| --- | --- | --- |"]
    for name in dict.fromkeys(list(REQUIREMENTS) + list(requirements)):
        row = requirements.get(name, {})
        detail = row.get("reason", row.get("scope", "" if row else "required evidence missing"))
        lines.append(f"| {cell(name)} | {cell(row.get('state', 'unproved'))} | {cell(detail)} |")
    lines += ["", "## Every timing mode", "",
              "Durations are medians in milliseconds. Ratios are candidate / baseline; the limit is 1.000.",
              "Upper bounds and paired ratios are taken from the report without dropping outliers or recomputing decisions.", "",
              "| Mode | State | Pairs | Baseline ms | Lustr ms | Paired ratio | Upper 95% | Worst pair | Detail |",
              "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |"]
    for name in dict.fromkeys(required + list(series)):
        row = series.get(name, {})
        reason = row.get("reason", "" if row else "required measurements missing")
        lines.append(f"| {cell(name)} | {cell(row.get('state', 'unproved'))} | {cell(row.get('pairs', 0))} | "
                     f"{number(row.get('baseline_median_ns'), 1e6)} | {number(row.get('candidate_median_ns'), 1e6)} | "
                     f"{number(row.get('geometric_ratio'))} | {number(row.get('upper_95'))} | "
                     f"{number(row.get('worst_observed_ratio'))} | {cell(reason)} |")
    lines += ["", "## Blocking requirements", ""]
    blocking = list(decision.get("blocking", []))
    blocking += [name for name in REQUIREMENTS if name not in requirements and name not in blocking]
    blocking += [name for name in required if name not in series and name not in blocking]
    lines += ["- " + cell(name) for name in blocking] or ["No blockers recorded." if qualified else "Acceptance is incomplete."]
    lines += ["", "## Retained execution failures", ""]
    failures = report.get("failures", [])
    lines += [f"- {cell(row.get('scope', 'unknown'))}: {cell(row.get('error', 'unknown failure'))}" for row in failures] or ["No execution failures recorded."]
    lines += ["", "Raw JSON, command output, traces and build receipts are retained in the workflow evidence artifact.", ""]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--receipt", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--source-commit", default="")
    args = parser.parse_args()
    report, receipt, steps, errors = None, None, None, []
    for label, path in (("report", args.report), ("receipt", args.receipt)):
        if path is None:
            continue
        try:
            record = load_record(path)
            if label == "report":
                report = record
            else:
                receipt = record
        except (OSError, ValueError, EOFError, zlib.error) as error:
            errors.append(f"Cannot read {label}: {error}")
    try:
        steps = json.loads(os.environ.get("LUSTR_CI_STEPS", "{}"))
        if not isinstance(steps, dict):
            raise ValueError("workflow steps must be an object")
        text = render(report, receipt, steps, args.source_commit, errors)
    except (ValueError, TypeError, AttributeError) as error:
        errors.append(f"Invalid evidence structure: {error}")
        text = render(None, source_commit=args.source_commit, errors=errors)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(text)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with Path(os.environ["GITHUB_STEP_SUMMARY"]).open("a") as stream:
            stream.write(text)
    print(f"Native qualification summary: {args.output}")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
