#!/usr/bin/env python3
"""Offline checks for the read-only upstream freshness gate."""
from __future__ import annotations

from contextlib import redirect_stdout
import importlib.util
import io
import json
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import MagicMock, patch


SCRIPT = Path(__file__).with_name("check-native-upstream-head.py")
SPEC = importlib.util.spec_from_file_location("native_upstream_head", SCRIPT)
head = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(head)
REVISION = "240e08d91d4aed0299639f6f11fd522912b248f9"
VALID = f"ref: refs/heads/next\tHEAD\n{REVISION}\tHEAD\n".encode()
PIN = (f'repository = "{head.UPSTREAM}"\nrevision = "{REVISION}"\n'
       'tree = "0ab1b09815802894e6078de0af59273cbdf46cb0"\npath = "compiler/native"\n').encode()


class HeadTests(unittest.TestCase):
    def test_exact_default_branch_and_commit(self):
        self.assertEqual(head.parse_head(VALID), {"default_ref": "refs/heads/next", "revision": REVISION})
        self.assertEqual(head.parse_head(b"\n".join(reversed(VALID.splitlines())))["revision"], REVISION)
        self.assertEqual(head.parse_head(VALID.replace(b"refs/heads/next", b"refs/heads/release/v1"))["default_ref"],
                         "refs/heads/release/v1")

    def test_missing_ambiguous_and_malformed_output(self):
        bad = [b"", VALID.splitlines()[0], VALID.splitlines()[1], VALID + VALID,
               VALID + f"{'b'*40}\tHEAD\n".encode(), VALID + b"ref: refs/heads/main\tHEAD\n",
               VALID.replace(REVISION.encode(), b"bad-sha"), VALID + b"unexpected\n",
               VALID.replace(b"refs/heads/next", b"refs/tags/next"), b"\xff"]
        for data in bad:
            with self.subTest(data=data), self.assertRaises(ValueError):
                head.parse_head(data)

    def test_invalid_branch_refs(self):
        for ref in ("refs/heads/", "refs/heads/a..b", "refs/heads/.hidden", "refs/heads/a.lock",
                    "refs/heads/a@{b", "refs/heads/a?b", "refs/heads/a//b", "refs/heads/a.",
                    "refs/heads/a\\b", "refs/heads/a b"):
            with self.subTest(ref=ref), self.assertRaises(ValueError):
                head.parse_head(VALID.replace(b"refs/heads/next", ref.encode()))

    def invoke(self, stdout=VALID, stderr=b"", code=0, timeout=False, spawn_error=None, pin=PIN):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            repository, output = root / "repo", root / "evidence"
            (repository / "upstream").mkdir(parents=True)
            (repository / "upstream/lust.pin").write_bytes(pin)
            process = MagicMock()
            process.__enter__.return_value = process
            process.returncode, process.pid = code, 12345
            process.communicate.side_effect = ([subprocess.TimeoutExpired("git", 20), (stdout, stderr)]
                                               if timeout else [(stdout, stderr)])
            with patch.object(head.subprocess, "Popen", side_effect=spawn_error, return_value=process) as run, \
                 patch.object(head.os, "killpg") as kill, \
                 patch.object(sys, "argv", [str(SCRIPT), "--repository", str(repository), "--output", str(output)]), \
                 redirect_stdout(io.StringIO()):
                status = head.main()
            record = json.loads((output / "report.json").read_text())
            self.assertEqual((repository / "upstream/lust.pin").read_bytes(), pin)
            self.assertEqual((output / "command.stdout").read_bytes(), stdout if run.called and not spawn_error else b"")
            self.assertEqual((output / "command.stderr").read_bytes(), stderr if run.called and not spawn_error else b"")
            self.assertEqual(record["helper_sha256"], head.digest(SCRIPT.read_bytes()))
            for stream in ("stdout", "stderr"):
                self.assertEqual(record["command"][stream + "_sha256"], head.digest((output / record["command"][stream]).read_bytes()))
            self.assertGreaterEqual(record["elapsed_ns"], 0)
            self.assertIn("started_at", record)
            if timeout:
                kill.assert_called_once_with(process.pid, signal.SIGKILL)
            else:
                kill.assert_not_called()
            if run.called:
                self.assertEqual(run.call_args.args[0], ["git", "ls-remote", "--symref", head.UPSTREAM, "HEAD"])
                env = run.call_args.kwargs["env"]
                self.assertEqual(env["GIT_TERMINAL_PROMPT"], "0")
                self.assertEqual(env["GIT_CONFIG_NOSYSTEM"], "1")
                if not spawn_error:
                    self.assertEqual(process.communicate.call_args_list[0].kwargs, {"timeout": 20})
            return status, record

    def test_matching_head_passes_and_retains_evidence(self):
        status, record = self.invoke()
        self.assertEqual(status, 0)
        self.assertTrue(record["success"])
        self.assertEqual(record["state"], "matched")
        self.assertEqual(record["pin"]["sha256"], head.digest(PIN))
        self.assertEqual(record["observed"]["revision"], REVISION)

    def test_drift_fails_without_changing_pin(self):
        status, record = self.invoke(stdout=VALID.replace(REVISION.encode(), b"b" * 40))
        self.assertEqual(status, 1)
        self.assertFalse(record["success"])
        self.assertEqual(record["state"], "drift")

    def test_missing_or_ambiguous_head_is_unavailable(self):
        for data in (b"", VALID + VALID):
            with self.subTest(data=data):
                status, record = self.invoke(stdout=data)
                self.assertEqual(status, 1)
                self.assertEqual(record["state"], "unavailable")

    def test_network_exit_and_spawn_errors_are_nonpassing(self):
        status, record = self.invoke(stdout=VALID, stderr=b"network unavailable\n", code=128)
        self.assertEqual(status, 1)
        self.assertEqual(record["command"]["exit_code"], 128)
        self.assertNotIn("observed", record)
        status, record = self.invoke(spawn_error=FileNotFoundError("git unavailable"))
        self.assertEqual(status, 1)
        self.assertIn("FileNotFoundError", record["errors"][0])

    def test_timeout_kills_query_group_and_preserves_partial_logs(self):
        status, record = self.invoke(stdout=b"partial\n", stderr=b"waiting\n", code=-9, timeout=True)
        self.assertEqual(status, 1)
        self.assertTrue(record["command"]["timed_out"])
        self.assertEqual(record["state"], "unavailable")

    def test_invalid_pin_never_queries_remote(self):
        for pin in (PIN.replace(REVISION.encode(), b"invalid"), PIN.replace(head.UPSTREAM.encode(), b"file:///elsewhere")):
            with self.subTest(pin=pin):
                status, record = self.invoke(pin=pin)
                self.assertEqual(status, 1)
                self.assertEqual(record["command"]["argv"], [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
