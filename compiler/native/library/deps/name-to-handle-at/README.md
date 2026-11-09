# Adapter: name-to-handle-at 0.1.0

Lust adapter crate (docs/ARCH.md section 3.6) for the whole
public API of the crate: `name_to_handle_at` and
`open_by_handle_at`, the `FileHandle` and `MountId` types,
and the four `AT_*` flag constants. Gate:
`scripts/test-adapters.sh name-to-handle-at`.

## Implemented, same results as upstream

- Both syscalls over the sysx intrinsics, with the real
  crate's sizing loop (zero-capacity probe, retry on
  EOVERFLOW), its `io::Error` from the raw errno, its
  interior-NUL path truncation at the kernel C string, its
  EINVAL for a handle longer than a u16 in
  `open_by_handle_at`, and its `MountId::Unique` selection
  by the `AT_HANDLE_MNT_ID_UNIQUE` flag bit.

## Known differences

- The real crate caches "syscall unsupported" in a static
  after an ENOSYS and skips later syscalls. The kernel
  answers ENOSYS on every call in that state, so results
  agree; only the number of syscalls differs.
