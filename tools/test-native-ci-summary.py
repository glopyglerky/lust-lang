#!/usr/bin/env python3
"""Check readable qualification summaries against real retained evidence."""
from __future__ import annotations

import copy
import gzip
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "tools/native-ci-summary.py"
SPEC = importlib.util.spec_from_file_location("native_ci_summary", SCRIPT)
summary = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(summary)
HISTORY = ROOT / "docs/compiler-set/measurements/2026-11-11-native-composition/optimized-ci/native-speed-evidence/report.json.gz"


class SummaryTests(unittest.TestCase):
    def test_required_summary_checks_have_workflow_ids(self):
        workflow = (ROOT / ".github/workflows/native-package.yml").read_text()
        ids = re.findall(r"^\s+id: ([a-z_]+)\s*$", workflow, re.M)
        self.assertEqual(len(ids), len(set(ids)))
        self.assertTrue(set(summary.WORKFLOW_CHECKS) <= set(ids))

    def complete(self):
        receipt = {"schema": "lustr.native-cargo-build.v2", "source": {"commit": "a" * 40},
                   "build": {"profile": "release", "settings": {"opt_level": 3, "codegen_units": 1,
                   "lto": "thin", "debug": "line-tables-only", "incremental": False}}}
        report = {"qualified": True, "decision": {"qualified": True}, "source": {"commit": "a" * 40},
                  "native_build_receipt": receipt,
                  "requirements": {name: {"state": "passed"} for name in summary.REQUIREMENTS},
                  "series": {name: {"state": "passed"} for name in summary.expected_series()}}
        steps = {name: {"outcome": "success"} for name in summary.WORKFLOW_CHECKS}
        return report, steps

    def test_historical_report_keeps_every_mode_and_failure(self):
        report = json.loads(gzip.decompress(HISTORY.read_bytes()))
        text = summary.render(report)
        self.assertIn("**UNQUALIFIED.", text)
        self.assertIn("Historical gate-profile build", text)
        self.assertIn("5.682", text)
        self.assertIn("1.100", text)
        self.assertIn("776346627bf7e29f10cd9352f7f12c59c5eea6d6bbfe4416630a7b57955c1721", text)
        for name, row in report["series"].items():
            self.assertIn(f"| {name} | {row['state']} |", text)
        for failure in report["failures"]:
            self.assertIn(summary.cell(failure["error"]), text)

    def test_missing_modes_are_unproved_even_if_report_claims_success(self):
        report = {"qualified": True, "decision": {"qualified": True},
                  "requirements": {"identities": {"state": "passed"}},
                  "series": {"lust/process-cold": {"state": "passed"}}}
        text = summary.render(report)
        self.assertIn("**UNQUALIFIED.", text)
        for name in summary.expected_series()[1:]:
            self.assertIn(f"| {name} | unproved |", text)

    def test_complete_report_is_not_called_universal(self):
        report, steps = self.complete()
        text = summary.render(report, steps=steps, source_commit="a" * 40)
        self.assertIn("**QUALIFIED for the recorded corpus and environment.**", text)
        self.assertIn("proves no universal", text)
        for key in ("functional_only", "diagnostic_without_trace"):
            diagnostic = dict(report, **{key: True})
            self.assertIn("**UNQUALIFIED.", summary.render(diagnostic, steps=steps, source_commit="a" * 40))

    def test_failed_cancelled_or_missing_correctness_step_blocks_success_headline(self):
        report, steps = self.complete()
        for state in ("failure", "cancelled", "skipped"):
            changed = copy.deepcopy(steps)
            changed["mixed_generics"] = {"outcome": state, "conclusion": "success"}
            text = summary.render(report, steps=changed, source_commit="a" * 40)
            self.assertIn("**UNQUALIFIED.", text)
            self.assertIn("Workflow check mixed_generics did not report success", text)
        del steps["artifacts"]
        self.assertIn("**UNQUALIFIED.", summary.render(report, steps=steps))

    def test_gate_receipt_and_wrong_release_settings_block_success_headline(self):
        report, steps = self.complete()
        self.assertIn("**UNQUALIFIED.", summary.render(report, receipt={}, steps=steps))
        for key, value in (("profile", "gate"), ("schema", "lustr.native-cargo-build.v1"),
                           ("settings", {"opt_level": 3, "codegen_units": 16})):
            changed = copy.deepcopy(report)
            target = changed["native_build_receipt"] if key == "schema" else changed["native_build_receipt"]["build"]
            target[key] = value
            text = summary.render(changed, steps=steps)
            self.assertIn("**UNQUALIFIED.", text)
            self.assertIn("required release build recipe", text)

    def test_requested_measured_and_receipt_sources_must_match(self):
        report, steps = self.complete()
        text = summary.render(report, steps=steps, source_commit="b" * 40)
        self.assertIn("**UNQUALIFIED.", text)
        self.assertIn("Requested source differs", text)
        report["native_build_receipt"]["source"]["commit"] = "b" * 40
        text = summary.render(report, steps=steps, source_commit="a" * 40)
        self.assertIn("**UNQUALIFIED.", text)
        self.assertIn("native receipt source are absent or different", text)

    def test_output_escapes_untrusted_diagnostics_and_preserves_workflow_failure(self):
        text = summary.render({"failures": [{"scope": "test|injected", "error": "<script>\n[link](https://invalid) | `code`"}]},
                              steps={"native_speed": {"outcome": "failure", "conclusion": "success"}})
        self.assertIn("| native_speed | failure |", text)
        self.assertNotIn("<script>", text)
        self.assertIn("test\\|injected", text)
        self.assertIn("\\[link\\]", text)

    def test_missing_file_writes_failure_summary_and_returns_error(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            env = dict(os.environ, GITHUB_STEP_SUMMARY=str(root / "github.md"), LUSTR_CI_STEPS="{}")
            result = subprocess.run([sys.executable, SCRIPT, "--report", root / "missing.json", "--output", root / "summary.md"],
                                    env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            text = (root / "summary.md").read_text()
            self.assertIn("Cannot read report", text)
            self.assertIn("**UNQUALIFIED.", text)
            self.assertEqual(text, (root / "github.md").read_text())

    def test_compressed_historical_cli_is_read_only(self):
        before = HISTORY.read_bytes()
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "summary.md"
            env = dict(os.environ, GITHUB_STEP_SUMMARY="", LUSTR_CI_STEPS="{}")
            result = subprocess.run([sys.executable, SCRIPT, "--report", HISTORY, "--output", output],
                                    env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("lust/warm-noop", output.read_text())
        self.assertEqual(before, HISTORY.read_bytes())

    def test_truncated_gzip_still_writes_an_unqualified_summary(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            report = root / "bad.json.gz"
            report.write_bytes(gzip.compress(b'{"series": {}}')[:-4])
            result = subprocess.run([sys.executable, SCRIPT, "--report", report, "--output", root / "summary.md"],
                                    env=dict(os.environ, GITHUB_STEP_SUMMARY="", LUSTR_CI_STEPS="{}"), capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertIn("Cannot read report", (root / "summary.md").read_text())
            self.assertIn("**UNQUALIFIED.", (root / "summary.md").read_text())

    def test_malformed_report_still_produces_an_unqualified_summary(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            report = root / "bad.json"
            report.write_text('{"series": []}')
            result = subprocess.run([sys.executable, SCRIPT, "--report", report, "--output", root / "summary.md"],
                                    env=dict(os.environ, GITHUB_STEP_SUMMARY="", LUSTR_CI_STEPS="{}"), capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertIn("Invalid evidence structure", (root / "summary.md").read_text())


if __name__ == "__main__":
    unittest.main(verbosity=2)
