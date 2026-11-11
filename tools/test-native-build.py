#!/usr/bin/env python3
"""Require the measured native compiler to use the shipped release recipe."""
import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("native_build", ROOT / "tools/build-native-cargo.py")
BUILD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BUILD)
TARGET = Path("/owned/native-build")
BOOTSTRAP = "/toolchain/bin/rustc"


def release_argv():
    # Cargo 1.99.1's observed release command for this profile, with paths and
    # dependency-free compiler input reduced to the fields the receipt binds.
    return [BOOTSTRAP, "--crate-name", "lustr_native", "--edition=2024", "src/main.rs",
            "--crate-type", "bin", "--emit=dep-info,link", "-C", "opt-level=3",
            "-C", "lto=thin", "-C", "codegen-units=1", "-C", "debuginfo=line-tables-only",
            "--out-dir", str(TARGET / BUILD.HOST / "release/deps"), "--target", BUILD.HOST]


class ReleaseReceipt(unittest.TestCase):
    def test_observed_release_recipe_is_accepted(self):
        settings = BUILD.validate_invocation(release_argv(), TARGET, BOOTSTRAP)
        self.assertEqual(settings, {"opt_level": 3, "codegen_units": 1, "lto": "thin",
                                    "debug": "line-tables-only", "incremental": False})

    def test_correctness_gate_cannot_qualify_as_a_release_measurement(self):
        args = release_argv()
        args[args.index("codegen-units=1")] = "codegen-units=16"
        args[args.index("debuginfo=line-tables-only")] = "debuginfo=0"
        args[args.index(str(TARGET / BUILD.HOST / "release/deps"))] = str(TARGET / BUILD.HOST / "gate/deps")
        args.extend(["-C", "incremental=" + str(TARGET / BUILD.HOST / "gate/incremental")])
        with self.assertRaises(ValueError):
            BUILD.validate_invocation(args, TARGET, BOOTSTRAP)

    def test_profile_overrides_or_missing_flags_do_not_pass(self):
        for original, wrong in (("opt-level=3", "opt-level=1"), ("codegen-units=1", "codegen-units=16"),
                                ("lto=thin", "lto=off"), ("debuginfo=line-tables-only", "debuginfo=0")):
            args = release_argv()
            args[args.index(original)] = wrong
            with self.subTest(override=wrong), self.assertRaises(ValueError):
                BUILD.validate_invocation(args, TARGET, BOOTSTRAP)
        args = release_argv()
        at = args.index("debuginfo=line-tables-only")
        del args[at-1:at+1]
        with self.assertRaises(ValueError):
            BUILD.validate_invocation(args, TARGET, BOOTSTRAP)

    def test_incremental_compiler_build_cannot_masquerade_as_release(self):
        args = release_argv() + ["-C", "incremental=" + str(TARGET / BUILD.HOST / "release/incremental")]
        with self.assertRaises(ValueError):
            BUILD.validate_invocation(args, TARGET, BOOTSTRAP)

    def test_duplicate_or_ambiguous_codegen_options_reject(self):
        for extra in (["-C", "opt-level=3"], ["--codegen=codegen-units=1"], ["-C"]):
            with self.subTest(extra=extra), self.assertRaises(ValueError):
                BUILD.validate_invocation(release_argv() + extra, TARGET, BOOTSTRAP)

    def test_wrong_compiler_crate_or_target_rejects(self):
        for original, wrong in ((BOOTSTRAP, "/wrapper"), ("lustr_native", "other"),
                                ("src/main.rs", "src/other.rs"), (BUILD.HOST, "aarch64-unknown-linux-gnu")):
            args = release_argv()
            args[args.index(original)] = wrong
            with self.subTest(value=wrong), self.assertRaises(ValueError):
                BUILD.validate_invocation(args, TARGET, BOOTSTRAP)

    def test_build_environment_forces_release_settings(self):
        self.assertEqual(BUILD.PROFILE_ENV, {
            "CARGO_PROFILE_RELEASE_OPT_LEVEL": "3", "CARGO_PROFILE_RELEASE_CODEGEN_UNITS": "1",
            "CARGO_PROFILE_RELEASE_LTO": "thin", "CARGO_PROFILE_RELEASE_DEBUG": "line-tables-only",
            "CARGO_PROFILE_RELEASE_INCREMENTAL": "false", "CARGO_INCREMENTAL": "0"})

    def test_old_receipt_schema_is_distinct(self):
        self.assertEqual(BUILD.SCHEMA, "lustr.native-cargo-build.v2")


if __name__ == "__main__":
    unittest.main(verbosity=2)
