#!/usr/bin/env python3
"""Check Cargo invocation evidence, including the exact failed CI log.

The fixture is native-initial stderr from run 43024037936, job 130534126306,
artifact 13856907244 at source 0bda81337a528a8ab910208357b65693b2f06fbb.
Cargo and the traced native compilation succeeded; the old parser lost the
unquoted executable path after package relocation into a directory with spaces.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shlex
import sys
import unittest


REPOSITORY = Path(__file__).resolve().parents[1]
FIXTURE = REPOSITORY / "tests/native-library/cargo-invocations/ci-43024037936-native-initial.stderr"
CI_COMPILER = Path("/tmp/lustr-package-ckvh5agv/relocated package/bin/lustr-native")
CI_PROJECT = "/home/runner/work/lustr/lustr/dist/package-qualification-native-library"


def running(command: str) -> str:
    return f"     Running {chr(96)}{command}{chr(96)}\n"


class InvocationTests(unittest.TestCase):
    parse = None

    def test_exact_failed_ci_stderr(self) -> None:
        raw = FIXTURE.read_bytes()
        self.assertEqual(hashlib.sha256(raw).hexdigest(),
                         "0b5740cdd0944ab12726d6a2b8c4f2091b83901a10749835e735fd6a5ff04467")
        calls = self.parse(raw.decode(), CI_COMPILER)
        self.assertEqual(len(calls), 3)
        versions = {}
        for call in calls:
            argv, environment = call["argv"], call["environment"]
            self.assertEqual(argv[0], str(CI_COMPILER))
            self.assertEqual(environment["CARGO"], "/tmp/lustr-package-ckvh5agv/relocated package/bin/cargo")
            self.assertEqual(environment["CARGO_PKG_DESCRIPTION"], "")
            self.assertEqual(environment["LD_LIBRARY_PATH"], CI_PROJECT + "/native target/debug/deps")
            self.assertEqual(argv[argv.index("--out-dir") + 1], CI_PROJECT + "/native target/debug/deps")
            versions[environment["CARGO_PKG_VERSION"]] = call
        self.assertEqual(set(versions), {"0.0.0", "0.1.0", "0.2.0"})
        for version, source in (("0.1.0", "left/src/lib.rs"), ("0.2.0", "right/src/lib.rs")):
            self.assertEqual(versions[version]["environment"]["CARGO_CRATE_NAME"], "shared_symbol")
            self.assertIn(source, versions[version]["argv"])
        consumer = versions["0.0.0"]["argv"]
        externs = [consumer[index + 1] for index, item in enumerate(consumer) if item == "--extern"]
        self.assertEqual(externs, [
            "left=" + CI_PROJECT + "/native target/debug/deps/libshared_symbol-53b2719a6979a43e.rlib",
            "right=" + CI_PROJECT + "/native target/debug/deps/libshared_symbol-63e657272b011f06.rlib",
        ])

    def test_raw_path_preserves_repeated_spaces_quotes_and_backslashes(self) -> None:
        compiler = Path('/tmp/a  path/with "quotes" and \'apostrophe\'/back\\slash/lustr-native')
        command = ("DESCRIPTION='two  spaces and \"quotes\"' EMPTY= "
                   + str(compiler) + " --crate-name example --extern 'dep=/tmp/another  path/libdep.rlib'")
        self.assertEqual(self.parse(running(command), compiler), [{
            "argv": [str(compiler), "--crate-name", "example", "--extern", "dep=/tmp/another  path/libdep.rlib"],
            "environment": {"DESCRIPTION": 'two  spaces and "quotes"', "EMPTY": ""},
        }])

    def test_quoted_executable_and_arguments(self) -> None:
        compiler = Path('/tmp/a  path/with "quotes" and \'apostrophe\'/lustr-native')
        for executable in (shlex.quote(str(compiler)), json.dumps(str(compiler))):
            with self.subTest(executable=executable):
                command = "SETTING='a  b' " + executable + " --cfg 'feature=\"quoted\"' 'VAR=argument'"
                self.assertEqual(self.parse(running(command), compiler), [{
                    "argv": [str(compiler), "--cfg", 'feature="quoted"', "VAR=argument"],
                    "environment": {"SETTING": "a  b"},
                }])

    def test_plain_executable_without_environment(self) -> None:
        compiler = Path("/tmp/plain/lustr-native")
        for command, argv in ((str(compiler), [str(compiler)]),
                              (str(compiler) + " --version", [str(compiler), "--version"])):
            with self.subTest(command=command):
                self.assertEqual(self.parse(running(command), compiler), [{"argv": argv, "environment": {}}])

    def test_environment_value_is_not_an_executable(self) -> None:
        command = "HINT=" + shlex.quote(str(CI_COMPILER) + " --crate-name fake") + " /usr/bin/printf example"
        self.assertEqual(self.parse(running(command), CI_COMPILER), [])

    def test_reject_wrappers_other_executables_and_path_prefixes(self) -> None:
        for command in (
            "/usr/bin/env " + str(CI_COMPILER) + " --crate-name wrong",
            "/tmp/wrapper " + shlex.quote(str(CI_COMPILER)) + " --crate-name wrong",
            str(CI_COMPILER) + "-wrapper --crate-name wrong",
            str(CI_COMPILER) + "/child --crate-name wrong",
            "/other" + str(CI_COMPILER) + " --crate-name wrong",
            shlex.quote(str(CI_COMPILER) + "-wrapper") + " --crate-name wrong",
        ):
            with self.subTest(command=command):
                self.assertEqual(self.parse(running(command), CI_COMPILER), [])

    def test_malformed_matching_invocation_fails_explicitly(self) -> None:
        for command in ("BROKEN='unterminated " + str(CI_COMPILER),
                        str(CI_COMPILER) + " --cfg 'unterminated"):
            with self.subTest(command=command):
                with self.assertRaises(ValueError):
                    self.parse(running(command), CI_COMPILER)

    def test_only_cargo_running_records_are_parsed(self) -> None:
        self.assertEqual(self.parse("not a command: " + str(CI_COMPILER), CI_COMPILER), [])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner", type=Path, default=REPOSITORY / "tools/qualify-native-library.py",
                        help="qualification runner to check; useful for the retained failing baseline")
    args = parser.parse_args()
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location("native_library_invocation_tests", args.runner)
    if spec is None or spec.loader is None:
        raise ValueError("cannot load native library qualification runner")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    InvocationTests.parse = staticmethod(module.compiler_invocations)
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(InvocationTests)
    return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
