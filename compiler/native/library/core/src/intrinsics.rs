// `core::intrinsics`: the private intrinsic signatures (docs/ARCH.md
// section 3.2).
//
// The machine-checked registry is `INTRINSICS` in src/ty/lang.rs: name,
// signature, C symbol, and the shipped crates that may call each row.
// This file is the author-facing mirror for std writers. Only shipped
// crates may name these paths; a user crate that tries gets L0642.
//
// Implemented (callable from std source, or from the named adapter):
//
//     pub fn exit(code: i32) -> !;
//     pub fn alloc_error(size: usize) -> !;                    // std: handle_alloc_error's hook text, abort
//     pub fn xxh3_64(input: &[u8], seed: u64) -> u64;          // xxhash-rust
//     pub fn xxh3_128(input: &[u8], seed: u64) -> u128;        // xxhash-rust
//     pub fn ryu_js_fmt_f64(out: &mut String, value: f64);     // ryu-js
//     pub fn clock_now_raw() -> u128;      // std: CLOCK_MONOTONIC, lo tv_sec, hi tv_nsec
//     pub fn system_time_raw() -> u128;    // std: CLOCK_REALTIME, same packing
//     pub fn fmt_align(f: &mut Formatter<'_>) -> u32;  // core, std: 0 left 1 right 2 center 3 none
//     pub fn panic_hook_custom(on: bool);              // std: a set_hook hook is installed
//     pub fn panic_default_hook(file: &str, line: u32, col: u32, msg: &str);  // std: default hook text
//     pub fn char_case(c: u32, upper: bool) -> u128;  // core: case mapping packed (points lo, count hi)
//     pub fn sys_symlink(original: &[u8], link: &[u8]) -> i32;  // std: 0, errno, or -1 (NUL byte)
//
// The std platform crate (library/std/src/platform.rs, W-SHIM; C in
// src/runtime/sys.c):
//
//     pub fn sys_write_std(fd: i32, buf: &[u8]) -> i64;         // n or -errno
//     pub fn sys_flush_std(fd: i32) -> i32;                     // 0 or errno
//     pub fn sys_os_error_text(code: i32, out: &mut String);
//     pub fn sys_fmt_arguments_bytes(args: fmt::Arguments) -> Vec<u8>;
//
// The socket calls behind `std::net` (w-cinterop; C in
// src/runtime/sysnet.c, contracts in sysnet.h). An address crosses
// as one 48-byte record of 8-byte little-endian fields:
//
//     pub fn sys_net_resolve(host: &[u8], port: u64, out: &mut Vec<u8>) -> i32;
//     pub fn sys_net_tcp_bind(rec: &[u8], out: &mut Vec<u8>) -> i32;
//     pub fn sys_net_udp_bind(rec: &[u8], out: &mut Vec<u8>) -> i32;
//     pub fn sys_net_tcp_connect(rec: &[u8], out: &mut Vec<u8>) -> i32;
//     pub fn sys_net_tcp_connect_timeout(rec: &[u8], timeout_us: i64, out: &mut Vec<u8>) -> i32;  // -10000 = poll timeout
//     pub fn sys_net_connect_fd(fd: i32, rec: &[u8]) -> i32;
//     pub fn sys_net_accept(fd: i32, out: &mut Vec<u8>) -> i32;
//     pub fn sys_net_addr_of(fd: i32, peer: i32, out: &mut Vec<u8>) -> i32;
//     pub fn sys_net_shutdown(fd: i32, how: i32) -> i32;
//     pub fn sys_net_setopt(fd: i32, opt: i32, val: i64) -> i32;
//     pub fn sys_net_getopt(fd: i32, opt: i32) -> i64;
//     pub fn sys_net_sendto(fd: i32, buf: &[u8], rec: &[u8]) -> i64;
//     pub fn sys_net_recvfrom(fd: i32, max: u64, flags: i32, out: &mut Vec<u8>, addr: &mut Vec<u8>) -> i64;
//
// The Unix calls behind the rustix adapter (w-cinterop2; C in
// src/runtime/sysx.c, contracts in sysx.h), callable only from
// that adapter:
//
//     pub fn sys_x_kill(pid: i32, sig: i32) -> i32;
//     pub fn sys_x_waitid(idtype: i32, id: i32, options: i32, out: &mut Vec<u8>) -> i32;
//     pub fn sys_x_sigmask(how: i32, set: u64) -> i32;              // nix only
//     pub fn sys_x_getppid() -> i32;
//     pub fn sys_x_prctl_deathsig(sig: i32) -> i32;
//     pub fn sys_x_getrlimit(resource: i32, out: &mut Vec<u8>) -> i32;
//     pub fn sys_x_setrlimit(resource: i32, current: u64, maximum: u64) -> i32;
//     pub fn sys_x_openat(dirfd: i32, path: &[u8], flags: i32, mode: u32) -> i32;
//     pub fn sys_x_statat(dirfd: i32, path: &[u8], flags: i32, out: &mut Vec<u8>) -> i32;
//     pub fn sys_x_fstat(fd: i32, out: &mut Vec<u8>) -> i32;
//     pub fn sys_x_statx(dirfd: i32, path: &[u8], flags: i32, mask: u32, out: &mut Vec<u8>) -> i32;
//     pub fn sys_x_accessat(dirfd: i32, path: &[u8], mode: i32, flags: i32) -> i32;
//     pub fn sys_x_utimensat(dirfd: i32, path: &[u8], a_sec: i64, a_nsec: i64, m_sec: i64, m_nsec: i64, flags: i32) -> i32;
//     pub fn sys_x_mknodat(dirfd: i32, path: &[u8], mode: u32, dev: u64) -> i32;
//     pub fn sys_x_fcntl_getfl(fd: i32) -> i32;
//     pub fn sys_x_fcntl_setfl(fd: i32, flags: i32) -> i32;
//     pub fn sys_x_winsize(fd: i32, out: &mut Vec<u8>) -> i32;
//     pub fn sys_x_inotify_init1(flags: i32) -> i32;             // fd or -errno
//     pub fn sys_x_inotify_add_watch(fd: i32, path: &[u8], mask: u32) -> i32;  // wd or -errno
//     pub fn sys_x_inotify_rm_watch(fd: i32, wd: i32) -> i32;    // 0 or errno
//     pub fn sys_x_getdents(fd: i32, max: u64, out: &mut Vec<u8>) -> i64;      // for RawDir, not yet called
//     pub fn sys_x_statvfs(path: &[u8], out: &mut Vec<u8>) -> i32;
//     pub fn sys_x_fstatvfs(fd: i32, out: &mut Vec<u8>) -> i32;
//     pub fn sys_x_fanotify_init(flags: u32, event_f_flags: u32) -> i32;  // nix only; fd or -errno
//     pub fn sys_x_fanotify_mark(fd: i32, flags: u32, mask: u64, dirfd: i32, has_path: i32, path: &[u8]) -> i32;  // nix only
//
// Every other row of the section 3.2 table is registered with its
// signature and privilege list, but has no body yet: a privileged call
// rejects with L0640 until W-STD lands the wrapper and the runtime hook.
// Declarations gain real foreign-fn form when the checker learns foreign
// fns (section 3.7); until then this module holds no items on purpose,
// so the file still parses as ordinary lust source.
