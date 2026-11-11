#!/usr/bin/env python3
"""Exercise the real native compiler's Cargo metadata-query boundary."""

import argparse
import hashlib
import json
import pathlib
import subprocess
import sys


BLUEFE_QUERY = [
    "-", "--crate-name", "___", "--print=file-names", "-Zallow-features=",
    "-Wrust_2018_idioms", "-Wunused_lifetimes", "--crate-type", "bin",
    "--crate-type", "rlib", "--crate-type", "dylib", "--crate-type", "cdylib",
    "--crate-type", "staticlib", "--crate-type", "proc-macro", "--print=sysroot",
    "--print=split-debuginfo", "--print=crate-name", "--print=cfg", "-Wwarnings",
]
ANALYSIS_OPTIONS = {
    "-Zallow-features=", "-Wrust_2018_idioms", "-Wunused_lifetimes", "-Wwarnings",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", required=True, type=pathlib.Path)
    parser.add_argument("--work", required=True, type=pathlib.Path)
    args = parser.parse_args()
    native = args.native.resolve(strict=True)
    work = args.work.resolve()
    work.mkdir(parents=True, exist_ok=False)
    source = work / "main.rs"
    source.write_text("fn main() {}\n")
    previous_output = work / "existing-output"
    marker = b"previous output must survive a rejected invocation\n"
    report = {
        "schema": "lustr.native-protocol-tests.v1",
        "scope": "Compiler introspection and option rejection; not language or package qualification",
        "qualified": False,
        "compiler": str(native),
        "compiler_sha256": hashlib.sha256(native.read_bytes()).hexdigest(),
        "runner_sha256": hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
        "protocol_sources": [
            "https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_driver_impl/src/lib.rs",
            "https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/build_context/target_info.rs",
        ],
        "commands": [],
        "checks": [],
        "failures": [],
    }

    def run(label, argv):
        result = subprocess.run(
            [str(native), *argv], input="", text=True, capture_output=True,
            cwd=work, timeout=30,
        )
        row = {"label": label, "argv": [str(native), *argv], "exit_code": result.returncode,
               "stdout": result.stdout, "stderr": result.stderr}
        report["commands"].append(row)
        return result

    def check(label, condition):
        report["checks"].append({"label": label, "passed": bool(condition)})
        if not condition:
            report["failures"].append(label)

    try:
        plain = run("plain-query", [v for v in BLUEFE_QUERY if v not in ANALYSIS_OPTIONS])
        check("plain query succeeds", plain.returncode == 0)
        lines = plain.stdout.splitlines()
        check("native file names and debug modes remain truthful",
              len(lines) > 5 and lines[:2] == ["___", "lib___.rlib"]
              and lines[3:5] == ["off", "___"])
        check("native runtime root exists", len(lines) > 2 and pathlib.Path(lines[2]).is_dir())
        check("unsupported crate types are reported to Cargo",
              all(f"unsupported crate type `{kind}`" in plain.stderr
                  for kind in ["dylib", "cdylib", "staticlib", "proc-macro"]))

        query = run("bluefe-query", BLUEFE_QUERY)
        check("BlueFe query succeeds", query.returncode == 0)
        check("analysis modifiers do not change introspection",
              query.stdout == plain.stdout and query.stderr == plain.stderr)

        separated = [v for v in BLUEFE_QUERY if v not in ANALYSIS_OPTIONS]
        separated += ["-Z", "allow-features=", "-W", "rust_2018_idioms",
                      "--warn=unused_lifetimes", "--warn", "warnings"]
        split_query = run("separated-options-query", separated)
        check("split and long option spellings are equivalent",
              split_query.returncode == 0 and split_query.stdout == plain.stdout
              and split_query.stderr == plain.stderr)

        nonempty = run("nonempty-allow-features-query",
                       [v if v != "-Zallow-features=" else "-Zallow-features=rustc_private,proc_macro_span"
                        for v in BLUEFE_QUERY])
        check("a feature allow-list does not enable features during introspection",
              nonempty.returncode == 0 and nonempty.stdout == plain.stdout
              and nonempty.stderr == plain.stderr)

        for index, modifier in enumerate([
            ["-Zallow-features="], ["-Z", "allow-features=rustc_private"],
            ["-Wrust_2018_idioms"], ["-W", "unused_lifetimes"], ["--warn=warnings"],
        ]):
            previous_output.write_bytes(marker)
            result = run(f"compile-analysis-option-{index}",
                         [str(source), "--edition=2024", "--crate-name=contract", "-o",
                          str(previous_output), *modifier])
            check(f"analysis option {index} remains an explicit compile failure",
                  result.returncode == 2 and result.stderr.startswith("error: lustr-native:"))
            check(f"analysis option {index} preserves prior output", previous_output.read_bytes() == marker)

        for index, modifier in enumerate([
            ["-Zunknown-option=value"], ["-Wunknown_lint"],
            ["-Zallow-features=bad feature"], ["-Zallow-features=a,,b"],
            ["-Wunused_lifetimes=yes"], ["-Zallow-features"],
        ]):
            result = run(f"unsupported-query-option-{index}", ["--print=cfg", *modifier])
            check(f"unsupported query option {index} is rejected", result.returncode == 2)

        version = run("version", ["-vV"])
        check("compiler reports its own identity",
              version.returncode == 0 and version.stdout.startswith("lustr-native ")
              and "binary: lustr-native\n" in version.stdout
              and "native-full-rust: false" in version.stdout)
    except (OSError, subprocess.TimeoutExpired) as error:
        report["failures"].append(f"runner failure: {error}")
    report["success"] = not report["failures"]
    (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"success": report["success"], "commands": len(report["commands"]),
                      "checks": len(report["checks"]), "failures": report["failures"],
                      "report": str(work / "report.json")}, indent=2))
    return 0 if report["success"] else 1


if __name__ == "__main__":
    sys.exit(main())
