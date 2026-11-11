#!/usr/bin/env python3
"""Assemble a source-bound Linux package without changing either compiler."""
from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tarfile
import tomllib


def run(argv: list[str | Path], cwd: Path | None = None) -> str:
    return subprocess.check_output([str(arg) for arg in argv], cwd=cwd, text=True).strip()


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def check_symlinks(root: Path) -> None:
    for path in root.rglob("*"):
        if path.is_symlink():
            target = os.readlink(path)
            if os.path.isabs(target) or not path.resolve().is_relative_to(root.resolve()):
                raise ValueError(f"package symlink escapes its root: {path.relative_to(root)}")


def copy_executable(source: Path, destination: Path) -> None:
    if not source.is_file() or not os.access(source, os.X_OK):
        raise ValueError(f"missing executable: {source}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)


def archive_git(repository: Path, revision: str, destination: Path) -> None:
    with destination.open("wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as zipped:
            process = subprocess.Popen(
                ["git", "-C", str(repository), "archive", "--format=tar", revision],
                stdout=subprocess.PIPE,
            )
            assert process.stdout is not None
            shutil.copyfileobj(process.stdout, zipped)
            process.stdout.close()
            if process.wait() != 0:
                raise RuntimeError("could not archive the recorded Git source")


def archive_directory(source: Path, destination: Path) -> None:
    def normalized(info: tarfile.TarInfo) -> tarfile.TarInfo:
        info.uid = info.gid = info.mtime = 0
        info.uname = info.gname = ""
        return info

    with destination.open("wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as zipped:
            with tarfile.open(fileobj=zipped, mode="w|") as archive:
                for path in sorted(source.rglob("*")):
                    archive.add(path, arcname=str(path.relative_to(source)), recursive=False, filter=normalized)


def compatibility_provenance(origin: str, root: Path, compiler_version: str, cargo_version: str) -> dict:
    identity = dict(line.split(": ", 1) for line in compiler_version.splitlines() if ": " in line)
    if identity.get("host") != "x86_64-unknown-linux-gnu":
        raise ValueError("compatibility compiler host does not match the package")
    if origin == "upstream-rust":
        if identity.get("release") != "1.99.1" or (root / "bluefe-package.json").exists():
            raise ValueError("upstream-rust packaging requires the pinned official Rust 1.99.1 toolchain")
        return {"origin": "upstream-rust", "pinned_release": "1.99.1"}
    record_path = root / "bluefe-package.json"
    if not record_path.is_file():
        raise ValueError("BlueFe origin requires its source-and-qualification record")
    record = json.loads(record_path.read_text())
    source = record.get("source", {})
    qualification = record.get("qualification", {})
    if (record.get("schema") != "bluefe.pristine-rust-package.v1"
            or record.get("role") != "rustc-compatibility-component"
            or record.get("target") != identity["host"]
            or source.get("repository") != "glopyglerky/blueFe"
            or not re.fullmatch(r"[0-9a-f]{40}", source.get("commit", ""))
            or source.get("commit") != identity.get("commit-hash")):
        raise ValueError("BlueFe record does not match the actual compiler source and host")
    gates = ("relocated_sysroot", "cargo_workspace_build", "cargo_workspace_tests_and_doctests",
             "proc_macro", "build_script", "runtime", "panic_recovery_and_drop", "type_error_rejection")
    if any(qualification.get(gate) is not True for gate in gates):
        raise ValueError("BlueFe record is missing a required qualification gate")
    versions = qualification.get("versions", {})
    actual_versions = {"rustc": compiler_version, "cargo": cargo_version,
                       "rustdoc": run([root / "bin/rustdoc", "--version", "--verbose"])}
    if any(versions.get(name, "").strip() != value for name, value in actual_versions.items()):
        raise ValueError("BlueFe recorded versions differ from the supplied toolchain")
    actual_hashes = {name: sha256(root / "bin" / name) for name in actual_versions}
    if record.get("tool_sha256") != actual_hashes:
        raise ValueError("BlueFe compiler files differ from the qualified binaries")
    return {"origin": "bluefe", "record_sha256": sha256(record_path), "record": record}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--native-bin", type=Path, required=True)
    parser.add_argument("--native-cargo-bin", type=Path, required=True)
    parser.add_argument("--rust-root", type=Path, required=True)
    parser.add_argument("--vendor-dir", type=Path, required=True)
    parser.add_argument("--tcc-root", type=Path)
    parser.add_argument("--tinycc-source", type=Path)
    parser.add_argument("--compatibility-origin", choices=["upstream-rust", "bluefe"], default="upstream-rust")
    args = parser.parse_args()
    if (platform.system(), platform.machine()) != ("Linux", "x86_64"):
        parser.error("this package recipe is qualified only on Linux x86_64")
    repository = args.repository.resolve()
    native = repository / "compiler/native"
    output = args.output.resolve()
    rust_root = args.rust_root.resolve()
    tcc = (args.tcc_root or native / "third_party/tcc").resolve()
    tinycc = (args.tinycc_source or native / "third_party/tinycc").resolve()
    if output.exists():
        parser.error("output must be a new directory")
    input_roots = [rust_root, args.native_bin.resolve(), args.native_cargo_bin.resolve(),
                   args.vendor_dir.resolve(), tcc, tinycc, native, repository / ".git"]
    if any(output.is_relative_to(path) or path.is_relative_to(output) for path in input_roots):
        parser.error("output must not overlap an input or protected source directory")
    if output.is_relative_to(repository) and output.relative_to(repository).parts[0] != "dist":
        parser.error("packages within the source checkout must be written under dist/")
    if not args.vendor_dir.is_dir() or not any(args.vendor_dir.iterdir()):
        parser.error("--vendor-dir must contain cargo vendor output")
    pin = tomllib.loads((repository / "upstream/lust.pin").read_text())
    revision = run(["git", "rev-parse", "HEAD"], repository)
    native_tree = run(["git", "rev-parse", "HEAD:compiler/native"], repository)
    if native_tree != pin["tree"]:
        raise ValueError("native compiler does not match the pinned upstream tree")
    if run(["git", "status", "--porcelain=v1", "--untracked-files=all"], repository):
        raise ValueError("commit implementation and packaging inputs before binding the package to HEAD")
    for name in ("lustr", "cargo-lustr", "cargo-lust"):
        path = rust_root / "bin" / name
        if path.exists() or path.is_symlink():
            raise ValueError(f"compatibility input contains an excluded coordinator executable: {path}")
    if run([rust_root / "bin/rustc", "--print", "sysroot"]) != str(rust_root):
        raise ValueError("--rust-root must name the actual compiler sysroot, not a rustup proxy")
    compiler_version = run([rust_root / "bin/rustc", "-vV"])
    cargo_version = run([rust_root / "bin/cargo", "-Vv"])
    compatibility = compatibility_provenance(args.compatibility_origin, rust_root, compiler_version, cargo_version)
    native_cargo_version = run([args.native_cargo_bin.resolve(), "--version", "--verbose"])
    if not native_cargo_version.startswith("lustr-native ") or "native-full-rust: false" not in native_cargo_version:
        raise ValueError("--native-cargo-bin must be the explicit experimental native compiler")
    upstream_lock = tomllib.loads((native / "Cargo.lock").read_text())
    entry_lock = tomllib.loads((repository / "crates/lustr-native/Cargo.lock").read_text())
    dependency_id = lambda package: (package["name"], package["version"], package.get("source"), package.get("checksum"))
    available_dependencies = {dependency_id(package) for package in upstream_lock["package"] if "source" in package}
    entry_dependencies = {dependency_id(package) for package in entry_lock["package"] if "source" in package}
    if not entry_dependencies.issubset(available_dependencies):
        raise ValueError("native Cargo dependencies are missing from the retained upstream vendor sources")
    tcc_revision = run(["git", "rev-parse", "HEAD"], tinycc)
    if tcc_revision != run(["git", "rev-parse", "43c7708"], tinycc):
        raise ValueError("TinyCC does not match the upstream Lust build recipe")
    if not (tcc / "lib/tcc/libtcc1.a").is_file():
        raise ValueError("build the pinned TinyCC runtime before packaging")
    if run(["git", "diff", "--name-only", "HEAD"], tinycc):
        raise ValueError("TinyCC tracked source differs from its recorded revision")
    shutil.copytree(rust_root, output, symlinks=True)
    for name in ("lust", "lust-core"):
        copy_executable(args.native_bin / name, output / "bin" / name)
    copy_executable(args.native_cargo_bin, output / "bin/lustr-native")
    support = output / "lib/lust"
    (support / "tcc/lib").mkdir(parents=True)
    copy_executable(tcc / "bin/tcc", support / "tcc/bin/tcc")
    shutil.copytree(tcc / "lib/tcc", support / "tcc/lib/tcc", symlinks=True)
    shutil.copytree(native / "library", support / "library", symlinks=True)
    notices = output / "share/licenses/lustr"
    notices.mkdir(parents=True)
    shutil.copy2(repository / "LICENSE", notices / "Lustr-LICENSE")
    shutil.copy2(native / "LICENSE", notices / "Lust-LICENSE")
    shutil.copy2(tinycc / "COPYING", notices / "TinyCC-COPYING")
    for path in args.vendor_dir.rglob("*"):
        if path.is_file() and ("license" in path.name.lower() or path.name.lower().startswith("copying")):
            destination = notices / "dependencies" / path.relative_to(args.vendor_dir)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, destination)
    source = output / "share/lustr/source"
    source.mkdir(parents=True)
    archive_git(repository, revision, source / "lustr.tar.gz")
    archive_git(tinycc, tcc_revision, source / "tinycc.tar.gz")
    archive_directory(args.vendor_dir, source / "vendor.tar.gz")
    shutil.copy2(repository / "docs/compiler-set/native-package.md", output / "LUSTR-README.md")
    check_symlinks(output)
    system_requirements = [
        "Linux x86_64; qualified on Ubuntu 24.04",
        "C development headers and linker",
    ]
    if args.compatibility_origin == "bluefe":
        system_requirements.extend([
            "glibc 2.39 or newer, including the x86_64 ELF loader",
            "libstdc++.so.6 providing GLIBCXX_3.4.32 and libgcc_s.so.1",
            "OpenSSL 3 runtime: libssl.so.3 and libcrypto.so.3",
            "libcurl.so.4 and libz.so.1, with their distribution dependencies",
            "CA certificates for Cargo HTTPS registry and Git access",
        ])
    else:
        system_requirements.append("glibc compatible with Ubuntu 24.04")
    manifest = {
        "schema": "lustr.toolchain-package",
        "schema_version": 1,
        "host": "x86_64-unknown-linux-gnu",
        "source_revision": revision,
        "native_lust": True,
        "native_full_rust": False,
        "runtime_coordinator": False,
        "rust_compatibility": args.compatibility_origin,
        "rust_compatibility_provenance": compatibility,
        "rustc_verbose_version": compiler_version,
        "cargo_verbose_version": cargo_version,
        "lust_revision": pin["revision"],
        "lust_tree": native_tree,
        "tinycc_revision": tcc_revision,
        "native_version": run([output / "bin/lust", "--version"]),
        "native_profile": "release",
        "native_cargo": {
            "status": "experimental-binary-crates",
            "executable": "bin/lustr-native",
            "implementation": "same-process compilation from pinned upstream Lust modules",
            "verbose_version": native_cargo_version,
            "build_profile": "gate",
            "build_settings": {"opt_level": 3, "codegen_units": 16, "lto": "thin", "debug": False},
            "native_full_rust": False,
            "native_rlib": False,
            "proc_macros": False,
            "test_harness": False,
            "daemon_reuse": False,
            "qualification": "embedded native_cargo_qualification in package-qualification.json",
        },
        "qualification": "separate package-qualification.json and native-qualification evidence",
        "system_requirements": system_requirements,
        "known_limitations": [{
            "id": "native-external-input-cache-invalidation",
            "status": "unfixed-in-pinned-upstream",
            "lust_revision": pin["revision"],
            "affected_macros": ["env", "option_env", "include", "include_str", "include_bytes"],
            "trigger": "whitespace or comments between the macro name and ! can evade the whole-request cache guard",
            "impact": "a native invocation can reuse stale results after an environment variable or included file changes",
            "source": "compiler/native/src/driver/cache.rs::src_reads_outside",
            "regression_test": "actual_lust_invalidates_external_reads_with_comments_before_macro_bang",
            "workaround": {
                "scope": "only the affected native invocation; package defaults remain unchanged",
                "environment": {"LUST_NO_DAEMON": "1", "LUST_NO_CACHE": "1"},
                "performance_cost": "disables daemon reuse and disk cache reuse; warm native latency is not preserved",
            },
        }],
        "payload": [],
    }
    for path in sorted(output.rglob("*")):
        if path.is_symlink():
            manifest["payload"].append({"path": path.relative_to(output).as_posix(), "symlink": os.readlink(path)})
        elif path.is_file():
            manifest["payload"].append({
                "path": path.relative_to(output).as_posix(),
                "sha256": sha256(path),
                "size": path.stat().st_size,
                "mode": oct(path.stat().st_mode & 0o777),
            })
    (output / "lustr-toolchain.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps({
        "package": str(output), "source_revision": revision,
        "native_full_rust": False, "rust_compatibility": args.compatibility_origin,
        "files": len(manifest["payload"]),
    }))


if __name__ == "__main__":
    main()

