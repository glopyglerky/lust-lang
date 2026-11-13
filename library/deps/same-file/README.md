# Adapter: same-file 1.0.6

Lust adapter crate (docs/ARCH.md section 3.6) for the file
identity form ts-rust's `compile.rs` uses
(`same_file::is_same_file(a, b).unwrap_or(false)`). Gate:
`scripts/test-adapters.sh same-file`.

## Implemented, same results as upstream

The adapter makes the same calls in the same order as
upstream's `unix.rs`, so answers and `io::Error` values match:

- `is_same_file`: opens the first path read-only, stats the
  open file, then the second; equality is device and inode
  number equality. `std::os::unix::fs::MetadataExt` (added to
  lust std for this adapter) reads the same stat fields.
- `Handle::from_path`, `Handle::from_file`, `as_file`,
  `as_file_mut`, `dev`, and `ino`. A handle keeps its file
  open until it drops. `Eq`, `PartialEq`, and `Hash` use the
  device and inode pair only, as upstream. The private field
  shape (`file`, `is_std`, `dev`, `ino`) mirrors upstream, so
  the derived Debug text has the same form; the device and
  inode values in it are the file system's own.

## Rejected (lust error, never a different answer)

- `Handle::stdin`, `Handle::stdout`, and `Handle::stderr`:
  upstream wraps fds 0, 1, and 2 in a `File` it must not
  close. Lust std has no from-raw-fd File form yet, and
  opening `/proc/self/fd/N` again is not the same open (it
  can fail or describe a different one for pipes), so the
  constructors are absent until std gains the fd types.
- The `AsRawFd` and `IntoRawFd` impls for `Handle`: lust std
  has no `std::os::fd` traits yet.
- Layout queries on `Handle` (no exact-layout entry): the
  fields mirror upstream for Debug parity, but the size is
  not promised to equal the real crate's.
