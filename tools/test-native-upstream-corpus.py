#!/usr/bin/env python3
"""Focused checks for bounded native/upstream corpus evidence handling."""
from __future__ import annotations

import importlib.util
import errno
import os
from pathlib import Path
import signal
import sys
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).with_name("check-native-upstream-corpus.py")
SPEC = importlib.util.spec_from_file_location("native_upstream_corpus", SCRIPT)
corpus = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(corpus)


class CorpusTests(unittest.TestCase):
    def test_inventory_includes_support_files_without_selecting_them_as_cases(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for suite in corpus.PROFILES:
                directory = root / "tests" / suite
                directory.mkdir(parents=True)
                (directory / "main.rs").write_text("fn main() {}\n")
            support = root / "tests/pass/module"
            support.mkdir()
            (support / "lib.rs").write_text("pub fn x() {}\n")
            (support / "data.bin").write_bytes(bytes(range(256)))
            cases, before = corpus.inventory(root)
            self.assertEqual([case["id"] for case in cases], ["pass/main.rs", "unwind/main.rs", "release/main.rs"])
            self.assertIn("tests/pass/module/lib.rs", before)
            self.assertIn("tests/pass/module/data.bin", before)
            (support / "data.bin").write_bytes(b"changed include data")
            self.assertNotEqual(before, corpus.inventory(root)[1])

    def test_empty_suite_and_invalid_selection_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(ValueError, "empty"):
                corpus.inventory(Path(temporary))
        cases = [{"id": "pass/a.rs"}, {"id": "pass/b.rs"}]
        self.assertEqual(corpus.select_cases(cases, []), cases)
        self.assertEqual(corpus.select_cases(cases, ["pass/b.rs"]), cases[1:])
        for selection in (["pass/a.rs", "pass/a.rs"], ["pass/missing.rs"], ["a"]):
            with self.assertRaises(ValueError):
                corpus.select_cases(cases, selection)

    def test_profile_flags_preserve_upstream_semantic_modes(self):
        self.assertEqual(corpus.profile_flags("pass"), corpus.profile_flags("unwind"))
        self.assertEqual(corpus.profile_flags("release"),
                         ["-C", "panic=unwind", "-C", "overflow-checks=off", "-C", "debug-assertions=off"])
        self.assertIn("overflow-checks=on", corpus.profile_flags("pass"))
        self.assertIn("debug-assertions=on", corpus.profile_flags("pass"))

    def test_masks_match_the_donor_rules_and_keep_other_differences(self):
        source, donor = Path("/work/lust/tests/pass/a.rs"), Path("/work/lust")
        raw = (b"thread 'worker' (1234) panicked at /work/lust/tests/pass/a.rs:8:3:\n"
               b"/rustc/ab12/library/core/src/panicking.rs:1:2\n"
               b"/toolchain/lib/rustlib/src/rust/library/std/src/lib.rs\n"
               b"stack backtrace:\n   0: frame\n             at location\n"
               b"timeout: the monitored command dumped core\n"
               b"case: line 42:  123 Aborted timeout program\n"
               b"address=0x123 pid=1234 value=17\n")
        masked = corpus.mask_output(raw, source, donor, stderr=True)
        self.assertEqual(masked,
                         b"thread 'worker' (TID) panicked at @CASE@:8:3:\n"
                         b"@STD@/library/core/src/panicking.rs:1:2\n"
                         b"@STD@/library/std/src/lib.rs\nstack backtrace:\n"
                         b"address=0x123 pid=1234 value=17\n")
        unusual = b"x\r   1: keep this\vtoo\ntimeout: the monitored command dumped core\r\n"
        self.assertEqual(corpus.mask_output(unusual, source, donor, stderr=True), unusual)
        self.assertEqual(corpus.mask_output(b"kept\n   0: removed", source, donor, stderr=True), b"kept\n")
        self.assertEqual(corpus.mask_output(b"/work/lust/tests/pass/a.rs /work/lust/a /work/lust/tests/pass/aXrs", source, donor, stderr=False),
                         b"@CASE@ @ROOT@/a @ROOT@/tests/pass/aXrs")

    def test_timeout_and_signals_are_retained_without_retry(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            command = corpus.execute([sys.executable, "-c", "import sys,time; print('before',flush=True); time.sleep(10)"],
                                     root, dict(os.environ), root, "timeout", 0.2, root)
            self.assertTrue(command["timed_out"])
            self.assertEqual(command["signal"], signal.SIGKILL)
            self.assertEqual((root / command["stdout"]).read_bytes(), b"before\n")
            self.assertFalse(corpus.complete_command(command))
            command = corpus.execute([sys.executable, "-c", "import os,signal; os.kill(os.getpid(),signal.SIGTERM)"],
                                     root, dict(os.environ), root, "signal", 5, root)
            self.assertEqual(command["exit_code"], -signal.SIGTERM)
            self.assertEqual(command["signal"], signal.SIGTERM)
            self.assertTrue(corpus.complete_command(command))
            missing = corpus.execute([str(root / "missing")], root, dict(os.environ), root, "missing", 1, root)
            self.assertIsNotNone(missing["error"])
            self.assertFalse(corpus.complete_command(missing))

    def test_matching_nonzero_is_valid_but_signal_and_output_mismatches_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            lanes = {}
            for lane in ("lust", "native"):
                (root / f"{lane}.stdout").write_bytes(b"result\n")
                (root / f"{lane}.stderr").write_bytes(b"panic expected\n")
                lanes[lane] = {"run": {"exit_code": 7, "stdout": f"{lane}.stdout", "stderr": f"{lane}.stderr"}}
            self.assertEqual(corpus.compare_runs(lanes, root, root / "case.rs", root, b"expected"), [])
            lanes["native"]["run"]["exit_code"] = -signal.SIGABRT
            self.assertIn("runtime exit status or signal differs", corpus.compare_runs(lanes, root, root / "case.rs", root, None))
            lanes["lust"]["run"]["exit_code"] = -signal.SIGABRT
            self.assertEqual(corpus.compare_runs(lanes, root, root / "case.rs", root, None), [])
            (root / "native.stdout").write_bytes(b"result=18\n")
            self.assertIn("runtime stdout differs after upstream masking", corpus.compare_runs(lanes, root, root / "case.rs", root, None))
            self.assertIn("native stderr lacks the fixture's panic expectation", corpus.compare_runs(lanes, root, root / "case.rs", root, b"missing"))

    def test_summary_never_promotes_missing_duplicate_partial_or_bounded_evidence(self):
        selected = [{"id": "pass/a.rs"}, {"id": "unwind/b.rs"}]
        rows = [{"id": case["id"], "state": "passed", "lanes": {"lust": {"compile": {}}, "native": {"compile": {}}}}
                for case in selected]
        bounded = corpus.summarize(rows, selected, 2, True)
        self.assertTrue(bounded["success"])
        self.assertFalse(bounded["qualified"])
        self.assertEqual(bounded["upstream_accepted_corpus"], "unproved")
        partial = corpus.summarize(rows, selected, 2293, True)
        self.assertEqual(partial["scope"], "explicit partial selection")
        self.assertFalse(partial["qualified"])
        no_second_compiler = [dict(rows[0], lanes={"lust": {"compile": {}}}), rows[1]]
        incomplete = corpus.summarize(no_second_compiler, selected, 2, True)
        self.assertFalse(incomplete["complete"])
        self.assertEqual(incomplete["compiler_pairs_attempted"], 1)
        self.assertEqual(incomplete["compiler_pairs_missing"], ["pass/a.rs"])
        for observed, stable in ((rows[:1], True), (rows + rows[:1], True),
                                 (rows + [{"id": "unselected", "state": "passed"}], True), (rows, False)):
            self.assertFalse(corpus.summarize(observed, selected, 2, stable)["success"])
        self.assertEqual(corpus.summarize([dict(rows[0], state="failed"), rows[1]], selected, 2, True)["state"], "failed")
        self.assertEqual(corpus.summarize([dict(rows[0], state="unproved"), rows[1]], selected, 2, True)["state"], "unproved")

    def test_sandbox_reset_preserves_equal_paths_and_declared_environment(self):
        with tempfile.TemporaryDirectory() as temporary:
            sandbox = Path(temporary) / "sandbox"
            corpus.reset_sandbox(sandbox)
            before = corpus.case_environment(sandbox)
            (sandbox / "side-effect").write_text("old")
            (sandbox / "tmp/data").write_text("old")
            corpus.reset_sandbox(sandbox)
            self.assertEqual(before, corpus.case_environment(sandbox))
            self.assertFalse((sandbox / "side-effect").exists())
            self.assertFalse((sandbox / "tmp/data").exists())
            self.assertEqual(before["LUST_NO_DAEMON"], "1")
            self.assertEqual(before["LUST_NO_CACHE"], "1")
            self.assertNotIn("LUST_DAEMON_SOCKET", before)

    def test_storage_exhaustion_preserves_both_lane_errors_without_claiming_a_compiler_regression(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "input.rs"
            source.write_text("fn main() {}\n")
            case = {"id": "pass/input.rs", "suite": "pass", "source": str(source), "source_sha256": corpus.sha(source)}
            compilers = {name: {"executable": "/unused"} for name in ("lust", "native")}
            with patch.object(corpus, "reset_sandbox", side_effect=OSError(errno.ENOSPC, "No space left on device")):
                row = corpus.run_case(case, 0, compilers, root, root, 1, 1)
            self.assertEqual(row["state"], "unproved")
            self.assertEqual(set(row["lanes"]), {"lust", "native"})
            self.assertTrue(all(lane["error_kind"] == "infrastructure" for lane in row["lanes"].values()))
            self.assertEqual(corpus.summarize([row], [case], 1, True)["compiler_pairs_attempted"], 0)

    def test_successful_compiler_exit_without_an_artifact_fails_the_contract(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "input.rs"
            source.write_text("fn main() {}\n")
            case = {"id": "pass/input.rs", "suite": "pass", "source": str(source), "source_sha256": corpus.sha(source)}
            compilers = {name: {"executable": "/unused"} for name in ("lust", "native")}
            command = {"timed_out": False, "error": None, "exit_code": 0}
            with patch.object(corpus, "execute", return_value=command) as execute:
                row = corpus.run_case(case, 0, compilers, root, root, 1, 1)
            self.assertEqual(execute.call_count, 2)
            self.assertEqual(row["state"], "failed")
            self.assertTrue(all(lane["error_kind"] == "artifact contract" for lane in row["lanes"].values()))


if __name__ == "__main__":
    unittest.main(verbosity=2)
