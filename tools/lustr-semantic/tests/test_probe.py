#!/usr/bin/env python3
"""Exercise the real pinned driver and artifacts; no compiler mocks for success."""

import argparse
import concurrent.futures
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
TOOLCHAIN = "nightly-2026-11-05"
COMMIT = "2d20b42b883dcfc014f3c2c56081eeb6091c2179"


def tool(name):
    return subprocess.check_output(
        ["rustup", "which", "--toolchain", TOOLCHAIN, name], text=True
    ).strip()


class ProbeTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="lustr-semantic-test-")
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.reports = self.directory / "reports"
        self.env = {**os.environ, "LUSTR_SEMANTIC_DIR": str(self.reports)}

    def run_command(self, args, *, expected=0, env=None):
        result = subprocess.run(
            args,
            cwd=self.directory,
            env=env or self.env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=120,
        )
        if expected is not None:
            self.assertEqual(
                result.returncode,
                expected,
                f"command {args!r}\nstdout:\n{result.stdout.decode(errors='replace')}"
                f"\nstderr:\n{result.stderr.decode(errors='replace')}",
            )
        return result

    def compile(self, arguments, *, inspect=True, expected=0):
        prefix = [PROBE, RUSTC] if inspect else [RUSTC]
        return self.run_command(prefix + ["--edition=2024", *map(str, arguments)], expected=expected)

    def load_reports(self):
        files = sorted(self.reports.glob("*.json"))
        documents = [json.loads(path.read_text()) for path in files]
        for document in documents:
            self.assertEqual(document["schema"], "lustr.rustc-semantic-inspection")
            self.assertEqual(document["schema_version"], 1)
            self.assertEqual(document["compiler"]["commit"], COMMIT)
            self.assertEqual(document["purpose"], "inspection-only")
            self.assertIs(document["compilation_succeeded"], True)
            self.assertIs(document["executable_ir"], False)
            self.assertIs(document["cache_key_complete"], False)
        self.assertEqual(list(self.reports.glob("*.tmp")), [])
        return documents

    @staticmethod
    def items(document):
        return [item for unit in document["codegen_units"] for item in unit["items"]]

    def fixture_arguments(self, name, output, extern=None):
        args = ["--crate-name", name, ROOT / "tests/fixtures" / f"{name}.rs", "-o", output]
        if name == "bridge":
            args += ["--crate-type", "rlib"]
        if extern is not None:
            args += ["--extern", f"bridge={extern}"]
        return args

    def test_artifacts_interoperate_both_directions_and_report_real_semantics(self):
        for inspected_side in ["library", "binary"]:
            with self.subTest(inspected_side=inspected_side):
                library = self.directory / f"libbridge_{inspected_side}.rlib"
                executable = self.directory / f"application_{inspected_side}"
                self.compile(self.fixture_arguments("bridge", library), inspect=inspected_side == "library")
                self.compile(
                    self.fixture_arguments("application", executable, library),
                    inspect=inspected_side == "binary",
                )
                output = self.run_command([executable])
                self.assertEqual(output.stdout, b"aggregate=3 drops=2 generics=30\n")
                self.assertEqual(output.stderr, b"")

        reports = {report["crate"]["name"]: report for report in self.load_reports()}
        self.assertEqual(set(reports), {"bridge", "application"})
        library_items = {item["definition"]: item for item in self.items(reports["bridge"])}
        self.assertEqual(library_items["large"]["abi"]["return"]["pass"]["kind"], "indirect")
        self.assertEqual(library_items["large"]["abi"]["return"]["layout"]["size_bytes"], 48)
        self.assertEqual(library_items["slice"]["abi"]["arguments"][0]["pass"]["kind"], "pair")
        self.assertEqual(library_items["invoke"]["abi"]["arguments"][0]["pass"]["kind"], "pair")
        niche = library_items["niche"]["abi"]["arguments"][0]["layout"]
        self.assertEqual(niche["size_bytes"], 8)
        self.assertEqual(niche["variants"]["encoding"]["kind"], "niche")

        application_items = self.items(reports["application"])
        generic = [item for item in application_items if item.get("definition") == "generic"]
        self.assertEqual({item["instance"] for item in generic}, {"generic::<u32>", "generic::<u64>"})
        self.assertEqual(len({item["symbol"] for item in generic}), 2)
        functions = [item for item in application_items if item["kind"] == "function"]
        self.assertTrue(any(item["drop_glue"] for item in functions))
        self.assertTrue(any(local["needs_drop"] for item in functions for local in item["mir"]["locals"]))
        blocks = [block for item in functions for block in item["mir"]["blocks"]]
        self.assertTrue(any(block["terminator"] == "drop" and "drop" in block for block in blocks))
        self.assertTrue(any(block["cleanup"] for block in blocks))
        self.assertTrue(any((block["unwind"] or {}).get("kind") == "cleanup" for block in blocks))
        self.assertEqual(reports["application"]["configuration"]["panic_strategy"], "Unwind")

    def test_analysis_only_does_not_force_monomorphization(self):
        source = self.directory / "recursive.rs"
        source.write_text(
            '#![recursion_limit = "16"]\n#![allow(unconditional_recursion)]\n'
            "fn recurse<T>(value: T) { recurse((value,)); }\n"
            "fn main() { recurse(1_u8); }\n"
        )
        self.compile([source, "--emit=metadata", "-o", "ordinary.rmeta"], inspect=False)
        self.compile([source, "--emit=metadata", "-o", "inspected.rmeta"])
        [report] = self.load_reports()
        self.assertEqual(report["configuration"]["mode"], "analysis-only")
        self.assertEqual(report["codegen_units"], [])
        # This fixture actually distinguishes type checking from codegen.
        normal = self.compile([source, "-o", "ordinary"], inspect=False, expected=1)
        inspected = self.compile([source, "-o", "inspected"], expected=1)
        self.assertIn(b"recursion limit", normal.stderr)
        self.assertIn(b"recursion limit", inspected.stderr)
        self.assertEqual(len(self.load_reports()), 1)

    def test_compile_and_late_link_failures_publish_nothing(self):
        invalid = self.directory / "invalid.rs"
        invalid.write_text("fn main() { missing_function(); }\n")
        args = [invalid, "--error-format=json", "-o", "invalid"]
        normal = self.compile(args, inspect=False, expected=1)
        inspected = self.compile(args, expected=1)
        self.assertEqual(inspected.stdout, normal.stdout)
        for result in [normal, inspected]:
            diagnostics = [json.loads(line) for line in result.stderr.splitlines()]
            self.assertTrue(any((entry.get("code") or {}).get("code") == "E0425" for entry in diagnostics))
        valid = self.directory / "valid.rs"
        valid.write_text("fn main() {}\n")
        arguments = [valid, "-Clinker=/does/not/exist/lustr-test-linker", "-o", "valid"]
        self.compile(arguments, inspect=False, expected=1)
        result = self.compile(arguments, expected=1)
        self.assertIn(b"linker", result.stderr)
        self.assertEqual(self.load_reports(), [])

    def test_panic_abort_and_codegen_options_are_preserved(self):
        self.compile(self.fixture_arguments("bridge", "libbridge.rlib") + ["-Cpanic=abort", "-Copt-level=1", "-Coverflow-checks=off"])
        [report] = self.load_reports()
        self.assertEqual(report["configuration"]["panic_strategy"], "Abort")
        self.assertEqual(report["configuration"]["opt_level"], "Less")
        self.assertIs(report["configuration"]["overflow_checks"], False)
        large = next(item for item in self.items(report) if item.get("definition") == "large")
        self.assertIs(large["abi"]["can_unwind"], False)

    def test_time_passes_total_is_preserved_and_print_queries_stay_clean(self):
        source = self.directory / "timed.rs"
        source.write_text("fn main() {}\n")
        flags = ["-Ztime-passes", "-Ztime-passes-format=json"]
        for inspect in [False, True]:
            result = self.compile([source, "-o", "timed", *flags], inspect=inspect)
            timing = [json.loads(line.removeprefix(b"time: ")) for line in result.stderr.splitlines() if line.startswith(b"time: ")]
            self.assertEqual(timing[-1]["pass"], "total")
            self.assertEqual(sum(entry["pass"] == "total" for entry in timing), 1)
        normal = self.compile(["--print=cfg", *flags], inspect=False)
        inspected = self.compile(["--print=cfg", *flags])
        self.assertEqual(inspected.stdout, normal.stdout)
        self.assertNotIn(b'"pass":"total"', inspected.stderr.replace(b" ", b""))
        self.assertEqual(len(self.load_reports()), 1)

    def test_compiler_identity_and_explicit_sysroot_mismatches_are_rejected(self):
        version = subprocess.check_output([RUSTC, "-vV"], text=True)
        sysroot = subprocess.check_output([RUSTC, "--print", "sysroot"], text=True).strip()
        other_sysroot = self.directory / "other-sysroot"
        other_sysroot.mkdir()
        for field in ["commit-hash", "host", "release", "sysroot"]:
            with self.subTest(field=field):
                fake_version = "\n".join(
                    f"{field}: mismatch" if line.startswith(f"{field}: ") else line
                    for line in version.splitlines()
                )
                fake_sysroot = str(other_sysroot) if field == "sysroot" else sysroot
                fake = self.directory / "compiler"
                fake.write_text(
                    f"#!{sys.executable}\nimport sys\n"
                    f"version = {fake_version!r}\nsysroot = {fake_sysroot!r}\n"
                    "if sys.argv[1:] == ['-vV']: print(version)\n"
                    "elif sys.argv[1:] == ['--print', 'sysroot']: print(sysroot)\n"
                    "else: raise SystemExit(93)\n"
                )
                fake.chmod(0o755)
                output = self.run_command([PROBE, str(fake), "--print", "cfg"], expected=1)
                self.assertIn(b"compiler ABI mismatch", output.stderr)
                self.assertEqual(output.stdout, b"")
        output = self.compile(["--sysroot", other_sysroot, "--print", "cfg"], expected=1)
        self.assertIn(b"compiler ABI mismatch", output.stderr)
        self.assertEqual(self.load_reports(), [])

    def test_inspection_envelope_rejects_incompatible_or_incomplete_reports(self):
        self.compile(self.fixture_arguments("bridge", "libbridge.rlib"))
        [report] = self.load_reports()
        report_file = next(self.reports.glob("*.json"))
        self.assertEqual(self.run_command([PROBE, "--validate", report_file]).stdout, b"")
        mutations = [
            ("schema", "unknown"), ("schema_version", 2), ("executable_ir", True),
            ("cache_key_complete", True), ("compilation_succeeded", False),
            ("compiler.commit", "different"), ("compiler.host", "different"),
            ("compiler.sysroot", "/different/installation"),
            ("configuration.mode", "analysis-only"),
        ]
        candidate = self.directory / "candidate.json"
        for field, value in mutations:
            with self.subTest(field=field):
                invalid = copy.deepcopy(report)
                target = invalid
                parts = field.split(".")
                for part in parts[:-1]:
                    target = target[part]
                target[parts[-1]] = value
                candidate.write_text(json.dumps(invalid))
                result = self.run_command([PROBE, "--validate", candidate], expected=1)
                self.assertIn(b"incompatible inspection envelope", result.stderr)
                self.assertEqual(result.stdout, b"")
        for change in ["missing", "unknown"]:
            invalid = copy.deepcopy(report)
            if change == "missing":
                del invalid["configuration"]
            else:
                invalid["unknown_field"] = True
            candidate.write_text(json.dumps(invalid))
            self.run_command([PROBE, "--validate", candidate], expected=1)

    @unittest.skipUnless(os.name == "posix", "executable wrapper fixture uses Unix scripts")
    def test_compiler_wrappers_cannot_masquerade_as_the_pinned_entrypoint(self):
        source = self.directory / "requires_wrapper.rs"
        source.write_text('#[cfg(not(wrapper_cfg))] compile_error!("wrapper cfg missing");\nfn main() {}\n')
        wrapper = self.directory / "compiler-wrapper"
        wrapper.write_text(
            f"#!{sys.executable}\nimport os, sys\nrustc = {RUSTC!r}\n"
            "args = sys.argv[1:]\n"
            "if args not in [['-vV'], ['--print', 'sysroot']]: args += ['--cfg', 'wrapper_cfg']\n"
            "os.execv(rustc, [rustc, *args])\n"
        )
        wrapper.chmod(0o755)
        args = ["--edition=2024", str(source), "-o", str(self.directory / "wrapped-app")]
        self.run_command([wrapper, *args])
        self.run_command([self.directory / "wrapped-app"])
        alias = self.directory / "alias"
        alias.symlink_to(wrapper)
        path_entry = self.directory / "bin"
        path_entry.mkdir()
        (path_entry / "rustc").symlink_to(wrapper)
        env = {**self.env, "PATH": f"{path_entry}{os.pathsep}{self.env.get('PATH', '')}"}
        for compiler in [str(wrapper), str(alias), "rustc"]:
            with self.subTest(compiler=compiler):
                result = self.run_command([PROBE, compiler, *args], expected=1, env=env)
                self.assertIn(b"unsupported compiler entrypoint", result.stderr)
                self.assertNotIn(b"wrapper cfg missing", result.stderr)
                self.assertEqual(result.stdout, b"")
        self.assertEqual(self.load_reports(), [])
        # A symlink to the real compiler has the same entrypoint and is allowed.
        actual_alias = self.directory / "actual-rustc"
        actual_alias.symlink_to(RUSTC)
        self.run_command([PROBE, actual_alias, *args, "--cfg", "wrapper_cfg"])
        self.assertEqual(len(self.load_reports()), 1)

    def test_response_files_and_space_containing_arguments(self):
        source = self.directory / "source with spaces.rs"
        source.write_text('#[cfg(not(preserved))]\ncompile_error!("cfg argument lost");\nfn main() {}\n')
        response = self.directory / "arguments with spaces.txt"
        response.write_text(f"--crate-name\nspaces\n--cfg\npreserved\n{source}\n-o\noutput with spaces\n")
        result = self.compile([f"@{response}"])
        self.assertEqual(result.stdout, b"")
        self.run_command([self.directory / "output with spaces"])
        [report] = self.load_reports()
        self.assertIn(f"@{response}", report["invocation"]["arguments"])

    @unittest.skipUnless(os.name == "posix", "byte-preserving exec is a Unix contract")
    def test_non_utf8_arguments_are_delegated_intact_without_inspection(self):
        fake = self.directory / "compiler"
        fake.write_text(
            f"#!{sys.executable}\nimport os, sys\n"
            "sys.stdout.buffer.write(os.fsencode(sys.argv[1]))\nraise SystemExit(42)\n"
        )
        fake.chmod(0o755)
        result = self.run_command([os.fsencode(PROBE), os.fsencode(fake), b"non-utf8-\xff"], expected=42)
        self.assertEqual(result.stdout, b"non-utf8-\xff")
        self.assertEqual(self.load_reports(), [])

    def test_concurrent_reports_are_complete_and_do_not_overwrite(self):
        source = self.directory / "small.rs"
        source.write_text("pub fn answer() -> u64 { 42 }\n")

        def compile_one(index):
            self.compile([source, "--crate-type=rlib", "--crate-name", f"small_{index}", "-o", f"small_{index}.rlib"])

        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
            list(executor.map(compile_one, range(4)))
        self.assertEqual({report["crate"]["name"] for report in self.load_reports()}, {f"small_{index}" for index in range(4)})

    def test_cargo_workspace_build_script_proc_macro_and_json_stream(self):
        # Cargo ignores target/ in build-script default change detection. Reports
        # elsewhere inside a package would themselves become changed inputs.
        self.reports = self.directory / "target/semantic-reports"
        self.env["LUSTR_SEMANTIC_DIR"] = str(self.reports)
        for name in ["src", "bridge/src", "macros/src"]:
            (self.directory / name).mkdir(parents=True)
        (self.directory / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["bridge", "macros"]\nresolver = "3"\n'
            '[package]\nname = "semantic-fixture"\nversion = "0.1.0"\nedition = "2024"\n'
            '[dependencies]\nbridge = { path = "bridge" }\nmacros = { path = "macros" }\n'
        )
        (self.directory / "bridge/Cargo.toml").write_text('[package]\nname = "bridge"\nversion = "0.1.0"\nedition = "2024"\n')
        shutil.copy(ROOT / "tests/fixtures/bridge.rs", self.directory / "bridge/src/lib.rs")
        (self.directory / "macros/Cargo.toml").write_text(
            '[package]\nname = "macros"\nversion = "0.1.0"\nedition = "2024"\n[lib]\nproc-macro = true\n'
        )
        (self.directory / "macros/src/lib.rs").write_text(
            'use proc_macro::TokenStream;\n#[proc_macro]\npub fn value(_: TokenStream) -> TokenStream { "31_u64".parse().unwrap() }\n'
        )
        (self.directory / "build.rs").write_text(
            'fn main() { let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()); '
            'std::fs::write(out.join("generated.rs"), "const GENERATED: u64 = 11;\\n").unwrap(); '
            'println!("cargo::rustc-cfg=semantic_fixture"); '
            'println!("cargo::rustc-check-cfg=cfg(semantic_fixture)"); }\n'
        )
        (self.directory / "src/main.rs").write_text(
            'include!(concat!(env!("OUT_DIR"), "/generated.rs"));\n'
            '#[cfg(not(semantic_fixture))] compile_error!("build script cfg missing");\n'
            'fn main() { println!("{}", bridge::identity(macros::value!()) + GENERATED); }\n'
        )
        env = {**self.env, "RUSTC": RUSTC, "RUSTC_WORKSPACE_WRAPPER": PROBE, "CARGO_TARGET_DIR": str(self.directory / "target")}
        env.pop("RUSTC_WRAPPER", None)
        env.pop("RUSTFLAGS", None)
        env.pop("CARGO_ENCODED_RUSTFLAGS", None)
        args = [CARGO, "build", "--offline", "--message-format=json"]
        result = self.run_command(args, env=env)
        messages = [json.loads(line) for line in result.stdout.splitlines()]
        self.assertTrue(any(message.get("reason") == "compiler-artifact" for message in messages))
        self.assertTrue(messages[-1]["success"])
        reports = self.load_reports()
        self.assertTrue({"semantic_fixture", "bridge", "macros", "build_script_build"}.issubset({report["crate"]["name"] for report in reports}))
        self.assertEqual(self.run_command([self.directory / "target/debug/semantic-fixture"]).stdout, b"42\n")
        self.run_command(args, env=env)
        self.assertEqual(len(self.load_reports()), len(reports), "a Cargo no-op must not compile again")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--probe", default=str(ROOT / "target/debug/lustr-semantic"))
    parser.add_argument("--rustc")
    parser.add_argument("--cargo")
    options, rest = parser.parse_known_args()
    PROBE = str(Path(options.probe).resolve())
    RUSTC = options.rustc or tool("rustc")
    CARGO = options.cargo or tool("cargo")
    unittest.main(argv=[sys.argv[0], *rest])
