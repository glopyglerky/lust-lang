#!/usr/bin/env python3
"""Retain an existing native build and its install-relative runtime inputs.

This uses the layout of scripts/release.sh without rebuilding or stripping the
executables. The archive, per-file hashes and build record identify the exact
compiler that a native comparison can reproduce.
"""
import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import tomllib


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def command(*args, cwd=None):
    return subprocess.check_output([str(arg) for arg in args], cwd=cwd, text=True).strip()


def copy_file(source, destination):
    if source.is_symlink() or not source.is_file():
        raise ValueError("expected a regular input file: " + str(source))
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    if sha(source) != sha(destination):
        raise ValueError("input changed while copying: " + str(source))


def copy_tree(source, destination):
    if source.is_symlink() or not source.is_dir():
        raise ValueError("expected a runtime directory: " + str(source))
    for path in sorted(source.rglob("*")):
        if path.is_symlink():
            raise ValueError("runtime symlinks are not packaged: " + str(path))
        if path.is_file():
            copy_file(path, destination / path.relative_to(source))


def git_identity(source):
    if command("git", "status", "--porcelain=v1", "--untracked-files=no", cwd=source):
        raise ValueError("source checkout has modified tracked files: " + str(source))
    return {"revision": command("git", "rev-parse", "HEAD", cwd=source),
            "tree": command("git", "rev-parse", "HEAD^{tree}", cwd=source),
            "commit_time": int(command("git", "log", "-1", "--format=%ct", cwd=source))}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--binary-dir", type=Path, required=True)
    parser.add_argument("--tcc-home", type=Path, required=True)
    parser.add_argument("--tcc-source", type=Path, required=True)
    parser.add_argument("--build-log", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    source, binaries = args.source.resolve(), args.binary_dir.resolve()
    tcc, tcc_source, output = args.tcc_home.resolve(), args.tcc_source.resolve(), args.output.resolve()
    archive = output.with_name(output.name + ".tar.gz")
    checksum = archive.with_name(archive.name + ".sha256")
    if output.exists() or archive.exists() or checksum.exists():
        parser.error("output directory, archive and checksum must be new")
    record = {"schema": "lust.native-build.v1", "source": git_identity(source),
              "tinycc": git_identity(tcc_source),
              "rustc": command("rustc", "-vV"), "cargo": command("cargo", "--version"),
              "release_profile": tomllib.loads((source / "Cargo.toml").read_text())["profile"]["release"],
              "package_script_sha256": sha(Path(__file__)),
              "purpose": "Exact native build inputs for reproduction; no qualification result is implied."}
    pin = re.search(r"^TCC_COMMIT=([0-9a-f]+)$", (source / "scripts/build-tcc.sh").read_text(), re.M)
    if pin is None or not record["tinycc"]["revision"].startswith(pin.group(1)):
        raise ValueError("TinyCC source does not match the compiler's pinned revision")
    for path in [binaries / "lust", binaries / "lust-core", tcc / "bin/tcc"]:
        if not os.access(path, os.X_OK):
            raise ValueError("missing executable: " + str(path))
    if not (tcc / "lib/tcc/libtcc1.a").is_file():
        raise ValueError("TinyCC runtime archive is missing")
    output.mkdir(parents=True)
    for name in ("lust", "lust-core"):
        copy_file(binaries / name, output / "bin" / name)
    copy_file(tcc / "bin/tcc", output / "lib/lust/tcc/bin/tcc")
    copy_tree(tcc / "lib/tcc", output / "lib/lust/tcc/lib/tcc")
    library_files = subprocess.check_output(["git", "ls-files", "-z", "--", "library"], cwd=source).decode().split("\0")
    for name in filter(None, library_files):
        copy_file(source / name, output / "lib/lust" / name)
    if not (output / "lib/lust/library/core/src/lib.rs").is_file():
        raise ValueError("native library source is incomplete")
    copy_file(source / "LICENSE", output / "LICENSE")
    copy_file(tcc_source / "COPYING", output / "licenses/tinycc/COPYING")
    for name in ("Cargo.toml", "Cargo.lock"):
        copy_file(source / name, output / "build" / name)
    copy_file(args.build_log, output / "build/compiler.log")

    metadata = json.loads(command("cargo", "metadata", "--format-version=1", "--locked", "--offline",
                                  "--manifest-path", source / "Cargo.toml"))
    record["dependency_licenses"] = []
    for package in sorted(metadata["packages"], key=lambda package: package["id"]):
        if package["source"] is None:
            continue
        root = Path(package["manifest_path"]).parent
        license_files = [path for path in root.rglob("*") if path.is_file()
                         and path.name.upper().startswith(("LICENSE", "COPYING", "NOTICE"))]
        if package.get("license_file"):
            license_files.append(root / package["license_file"])
        if not license_files:
            raise ValueError("dependency has no packaged license text: " + package["id"])
        relative = Path("licenses/dependencies") / (package["name"] + "-" + package["version"])
        for path in sorted(set(license_files)):
            copy_file(path, output / relative / path.relative_to(root))
        record["dependency_licenses"].append({"name": package["name"], "version": package["version"],
                                             "license": package["license"], "directory": str(relative)})
    rust_docs = Path(command("rustc", "--print", "sysroot")) / "share/doc/rust"
    copy_file(rust_docs / "COPYRIGHT-library.html", output / "licenses/rust/COPYRIGHT-library.html")
    copy_tree(rust_docs / "licenses", output / "licenses/rust/licenses")
    record["binaries"] = {name: sha(output / "bin" / name) for name in ("lust", "lust-core")}
    (output / "build/identity.json").write_text(json.dumps(record, indent=2) + "\n")
    files = sorted(path for path in output.rglob("*") if path.is_file())
    (output / "SHA256SUMS").write_text("".join(sha(path) + "  " + path.relative_to(output).as_posix() + "\n" for path in files))
    temporary = archive.with_name(archive.name + ".tmp")
    with temporary.open("wb") as raw, gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as compressed:
        with tarfile.open(fileobj=compressed, mode="w") as tar:
            for path in [output, *sorted(output.rglob("*"))]:
                info = tar.gettarinfo(str(path), arcname=str(Path(output.name) / path.relative_to(output)))
                info.uid = info.gid = 0
                info.uname = info.gname = ""
                info.mtime = record["source"]["commit_time"]
                if info.isfile():
                    with path.open("rb") as stream:
                        tar.addfile(info, stream)
                else:
                    tar.addfile(info)
    temporary.replace(archive)
    checksum.write_text(sha(archive) + "  " + archive.name + "\n")
    print(json.dumps({"archive": str(archive), "sha256": sha(archive), "source": record["source"],
                      "binaries": record["binaries"]}, indent=2))


if __name__ == "__main__":
    main()
