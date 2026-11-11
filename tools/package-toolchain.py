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
    parser.add_argument("--coordinator-bin", type=Path, required=True)
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
    input_roots = [rust_root, args.native_bin.resolve(), args.coordinator_bin.resolve(), args.vendor_dir.resolve(), tcc, tinycc, native, repository / ".git"]
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
    if run([rust_root / "bin/rustc", "--print", "sysroot"]) != str(rust_root):
        raise ValueError("--rust-root must name the actual compiler sysroot, not a rustup proxy")
    compiler_version = run([rust_root / "bin/rustc", "-vV"])
    cargo_version = run([rust_root / "bin/cargo", "-Vv"])
    compatibility = compatibility_provenance(args.compatibility_origin, rust_root, compiler_version, cargo_version)
    tcc_revision = run(["git", "rev-parse", "HEAD"], tinycc)
    if tcc_revision != run(["git", "rev-parse", "43c7708"], tinycc):
        raise ValueError("TinyCC does not match the upstream Lust build recipe")
    if not (tcc / "lib/tcc/libtcc1.a").is_file():
        raise ValueError("build the pinned TinyCC runtime before packaging")
    if run(["git", "diff", "--name-only", "HEAD"], tinycc):
        raise ValueError("TinyCC tracked source differs from its recorded revision")
    shutil.copytree(rust_root, output, symlinks=True)
    for name in ("lustr", "cargo-lustr", "cargo-lust"):
        copy_executable(args.coordinator_bin / name, output / "bin" / name)
    for name in ("lust", "lust-core"):
        copy_executable(args.native_bin / name, output / "bin" / name)
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
    manifest = {
        "schema": "lustr.toolchain-package",
        "schema_version": 1,
        "host": "x86_64-unknown-linux-gnu",
        "source_revision": revision,
        "native_lust": True,
        "native_full_rust": False,
        "rust_compatibility": args.compatibility_origin,
        "rust_compatibility_provenance": compatibility,
        "rustc_verbose_version": compiler_version,
        "cargo_verbose_version": cargo_version,
        "lust_revision": pin["revision"],
        "lust_tree": native_tree,
        "tinycc_revision": tcc_revision,
        "native_version": run([output / "bin/lust", "--version"]),
        "native_profile": "release",
        "qualification": "separate package-qualification.json and native-qualification evidence",
        "system_requirements": ["Linux x86_64", "glibc compatible with Ubuntu 24.04", "C development headers and linker"],
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

