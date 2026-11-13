/* lust_shim.h: C ABI of liblust_shim.so (lust W-SHIM, ARCH 3.7).
 *
 * One rustc-built cdylib exports the coarse system services that ts-rust
 * gets from rustix, nix, signal-hook, name-to-handle-at, flate2, jiff,
 * xxhash-rust, num-bigint, and tikv-jemalloc-ctl on Linux. Every export
 * exists because ts-rust calls it. README.md maps each export to its
 * call sites. This header is the only contract. No Rust type crosses it.
 *
 * Boundary rules (ARCH 3.7):
 * - Only scalars, byte buffers (ptr plus len), the C records below, raw
 *   fd numbers, and opaque u64 handles cross.
 * - Inputs are borrowed for the call. The shim never keeps an input
 *   pointer after it returns.
 * - Variable outputs are LustShimBuf values. The shim owns that memory.
 *   Free it only with lust_shim_buf_free. Never free it with the C
 *   allocator, and never pass C memory to a shim free function.
 * - Every fallible export returns a status (see below) and catches its
 *   own Rust panics. A panic never unwinds through C frames.
 * - Callbacks return an int status. The shim stops at the first
 *   non-zero status and never calls back again after it.
 * - No C _Thread_local is used on the C side of this contract. The
 *   last-error text lives in Rust thread-local storage inside the
 *   cdylib, which spikes/RESULTS.md K3-10 and K6 prove is safe.
 *
 * Status values (int32_t):
 *    0  success
 *   >0  an errno value from the failed syscall (use strerror)
 *   -1  LUST_SHIM_ERR_PANIC: a Rust panic was caught in the shim
 *   -2  LUST_SHIM_ERR_INVALID: a null, bad handle, or bad argument
 *   -3  LUST_SHIM_ERR: any other failure, text in the last error
 *   -4  LUST_SHIM_ERR_UNSUPPORTED: a feature this build does not have
 * On any failure, lust_shim_last_error gives a one-line message.
 *
 * File descriptors: exports that open an fd return its raw number and
 * keep the owning Rust object in a shim table under that number (the
 * shape ts-rust fswatch/unix.rs uses). Close such an fd only with
 * lust_shim_close. Read, write, poll, fstat, fcntl, and winsize also
 * accept fds the shim does not own (0, 1, 2, or C-created fds). They
 * borrow those numbers and never close them.
 *
 * Paths are byte strings (ptr plus len), not NUL-terminated text,
 * because Go strings in ts-rust can hold any OS bytes. A path with an
 * inner NUL byte fails with EINVAL (22), as Go's BytePtrFromString does.
 * dirfd -100 means the current directory (AT_FDCWD).
 *
 * ABI version 1. Check it with lust_shim_abi_version at start.
 */
#ifndef LUST_SHIM_H
#define LUST_SHIM_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define LUST_SHIM_ABI_VERSION 1u
#define LUST_SHIM_OK 0
#define LUST_SHIM_ERR_PANIC (-1)
#define LUST_SHIM_ERR_INVALID (-2)
#define LUST_SHIM_ERR (-3)
#define LUST_SHIM_ERR_UNSUPPORTED (-4)
#define LUST_SHIM_AT_FDCWD (-100)

/* A shim-owned byte buffer. Zero it before the first call. After a
 * successful call, ptr and len give the bytes. Free with
 * lust_shim_buf_free, which resets the struct to zero. cap is private. */
typedef struct {
    uint8_t *ptr;
    size_t len;
    size_t cap;
} LustShimBuf;

/* Core. */
uint32_t lust_shim_abi_version(void);
/* Copies this thread's last error text. Returns the full byte length.
 * Writes at most cap-1 bytes plus a NUL when cap > 0. */
size_t lust_shim_last_error(char *out, size_t cap);
void lust_shim_buf_free(LustShimBuf *buf);
/* Test support: panics inside the shim on purpose. Must return
 * LUST_SHIM_ERR_PANIC with the message "lust shim forced panic 42". */
int32_t lust_shim_panic_for_test(void);

/* Allocator bridge (ARCH 3.7). These route to the global allocator of
 * this cdylib: jemalloc in a --features jemalloc build, else the
 * system allocator. align must be a power of two. A null result means
 * the allocation failed. Free only with lust_shim_dealloc and the same
 * size and align. realloc keeps the old block on failure. */
int32_t lust_shim_alloc(size_t size, size_t align, uint8_t **out_ptr);
int32_t lust_shim_alloc_zeroed(size_t size, size_t align, uint8_t **out_ptr);
int32_t lust_shim_realloc(uint8_t *ptr, size_t old_size, size_t align,
                          size_t new_size, uint8_t **out_ptr);
void lust_shim_dealloc(uint8_t *ptr, size_t size, size_t align);

/* ------------------------------------------------------------------ */
/* File descriptors and files (rustix fs, io, pipe, event, termios).  */
/* Flag values are the raw Linux bits, as ts-rust passes them through */
/* rustix from_bits_retain. <fcntl.h> on the C side names them.       */
/* ------------------------------------------------------------------ */

/* fstat result, the fields fswatch Stat_t and osvfs read. */
typedef struct {
    uint64_t dev;
    uint64_t ino;
    uint64_t nlink;
    uint32_t mode;
    uint32_t uid;
    uint32_t gid;
    uint64_t rdev;
    int64_t size;
    int64_t blksize;
    int64_t blocks;
    int64_t atime_sec;
    int64_t atime_nsec;
    int64_t mtime_sec;
    int64_t mtime_nsec;
    int64_t ctime_sec;
    int64_t ctime_nsec;
} LustShimStat;

/* statx result for the one osvfs use (type plus mtime). mask is the
 * kernel stx_mask, so the caller can require the bits it asked for. */
typedef struct {
    uint32_t mask;
    uint32_t mode;
    int64_t mtime_sec;
    uint32_t mtime_nsec;
    uint32_t reserved;
} LustShimStatx;

/* statvfs result. fsid is val[0] | val[1] << 32, as rustix reports it. */
typedef struct {
    uint64_t bsize;
    uint64_t blocks;
    uint64_t bfree;
    uint64_t bavail;
    uint64_t files;
    uint64_t ffree;
    uint64_t fsid;
    uint64_t namemax;
    uint64_t frsize;
    uint64_t flag;
} LustShimStatvfs;

typedef struct {
    int32_t fd;
    int16_t events;
    int16_t revents;
} LustShimPollFd;

typedef struct {
    uint16_t row;
    uint16_t col;
    uint16_t xpixel;
    uint16_t ypixel;
} LustShimWinsize;

/* utimensat input. A nsec of LUST_SHIM_UTIME_OMIT keeps that time. */
#define LUST_SHIM_UTIME_OMIT 1073741822
#define LUST_SHIM_UTIME_NOW 1073741823
typedef struct {
    int64_t atime_sec;
    int64_t atime_nsec;
    int64_t mtime_sec;
    int64_t mtime_nsec;
} LustShimTimestamps;

/* readdir_each callback. name is borrowed for the call only. file_type
 * is the kernel d_type byte (DT_DIR 4, DT_REG 8, and so on). Return 0
 * to continue. Any other value stops the walk at once. */
typedef int (*LustShimDirentFn)(void *env, uint64_t ino, uint32_t file_type,
                                const uint8_t *name, size_t name_len);

int32_t lust_shim_pipe2(uint32_t flags, int32_t out_fds[2]);
int32_t lust_shim_read(int32_t fd, uint8_t *buf, size_t cap, size_t *out_n);
int32_t lust_shim_write(int32_t fd, const uint8_t *buf, size_t len, size_t *out_n);
int32_t lust_shim_close(int32_t fd);
int32_t lust_shim_open(const uint8_t *path, size_t path_len, int32_t flags,
                       uint32_t mode, int32_t *out_fd);
int32_t lust_shim_openat(int32_t dirfd, const uint8_t *path, size_t path_len,
                         int32_t flags, uint32_t mode, int32_t *out_fd);
int32_t lust_shim_fstat(int32_t fd, LustShimStat *out);
int32_t lust_shim_statat(int32_t dirfd, const uint8_t *path, size_t path_len,
                         int32_t flags, LustShimStat *out);
int32_t lust_shim_lstat(const uint8_t *path, size_t path_len, LustShimStat *out);
int32_t lust_shim_statx(int32_t dirfd, const uint8_t *path, size_t path_len,
                        int32_t flags, uint32_t mask, LustShimStatx *out);
int32_t lust_shim_fcntl_getfl(int32_t fd, uint32_t *out_flags);
int32_t lust_shim_accessat(int32_t dirfd, const uint8_t *path, size_t path_len,
                           uint32_t access, int32_t flags);
int32_t lust_shim_utimensat(int32_t dirfd, const uint8_t *path, size_t path_len,
                            const LustShimTimestamps *times, int32_t flags);
/* file_type_with_mode is the S_IFMT bits of the new node (S_IFIFO and
 * so on). ts-rust uses this only in a test, to make a fifo. */
int32_t lust_shim_mknodat(int32_t dirfd, const uint8_t *path, size_t path_len,
                          uint32_t file_type_with_mode, uint32_t mode, uint64_t dev);
int32_t lust_shim_statvfs(const uint8_t *path, size_t path_len, LustShimStatvfs *out);
/* One getdents64 batch in kernel linux_dirent64 layout (name at record
 * offset 19). cap_hint sizes the batch buffer. An empty buf at status
 * OK means the end of the directory. */
int32_t lust_shim_read_dirent(int32_t fd, size_t cap_hint, LustShimBuf *out);
/* One getdents pass over fd, one callback per entry (dot entries
 * included, as rustix RawDir yields them). When the callback stops the
 * walk, the status is still OK, out_cb_status holds its value, and
 * out_calls counts the calls made. */
int32_t lust_shim_readdir_each(int32_t fd, LustShimDirentFn cb, void *env,
                               int64_t *out_calls, int32_t *out_cb_status);
int32_t lust_shim_poll(LustShimPollFd *fds, size_t count, int32_t timeout_ms,
                       int32_t *out_n);
int32_t lust_shim_dup2_stdout(int32_t fd);
int32_t lust_shim_dup2_stderr(int32_t fd);
int32_t lust_shim_tcgetwinsize(int32_t fd, LustShimWinsize *out);

/* ------------------------------------------------------------------ */
/* Processes, limits, prctl (rustix process, nix process/signal).     */
/* Spawning stays with the caller (std::process in ts-rust, posix or  */
/* fork plus exec in C). These are the process calls ts-rust takes    */
/* from rustix and nix. Signal numbers are the host <signal.h> ones.  */
/* ------------------------------------------------------------------ */

typedef struct {
    uint64_t current;
    uint64_t maximum;
    int32_t current_is_none;
    int32_t maximum_is_none;
} LustShimRlimit;

/* The only resources ts-rust reads or writes. */
#define LUST_SHIM_RLIMIT_NOFILE 0u
#define LUST_SHIM_RLIMIT_AS 1u
#define LUST_SHIM_RLIMIT_DATA 2u

int32_t lust_shim_getpid(int32_t *out_pid);
int32_t lust_shim_getppid(int32_t *out_pid); /* 0 means no parent */
int32_t lust_shim_kill(int32_t pid, int32_t signal);
int32_t lust_shim_test_kill(int32_t pid);
/* waitid(P_PID, pid, WEXITED | WNOWAIT): waits for the child to exit
 * without reaping it. One call, no EINTR retry; the caller retries, as
 * ts-rust bin/tsgo.rs does. */
int32_t lust_shim_waitid_exited_nowait(int32_t pid);
/* signal 0 clears the parent-death signal. */
int32_t lust_shim_set_parent_death_signal(int32_t signal);
int32_t lust_shim_getrlimit(uint32_t resource, LustShimRlimit *out);
int32_t lust_shim_setrlimit(uint32_t resource, const LustShimRlimit *limit);
int32_t lust_shim_prctl_get_thp_disable(int32_t *out_disabled);
int32_t lust_shim_prctl_set_thp_disable(int32_t disabled);
/* Unblocks the given signals on this thread (nix SigSet thread_unblock,
 * the one mask call ts-rust makes). */
int32_t lust_shim_thread_unblock_signals(const int32_t *signals, size_t count);

/* ------------------------------------------------------------------ */
/* Signals (signal-hook). Handlers never run caller code: a flag is   */
/* an AtomicBool the shim owns, and an iterator is a self-pipe queue  */
/* the caller drains. Handles are opaque and never 0.                 */
/* ------------------------------------------------------------------ */

int32_t lust_shim_signal_flag_new(int32_t initial, uint64_t *out_handle);
int32_t lust_shim_signal_flag_get(uint64_t handle, int32_t *out_value);
int32_t lust_shim_signal_flag_set(uint64_t handle, int32_t value);
int32_t lust_shim_signal_flag_free(uint64_t handle);
/* signal_hook flag::register: the signal sets the flag to true. */
int32_t lust_shim_signal_register_flag(int32_t signal, uint64_t flag);
/* flag::register_conditional_default: while the flag is true, the
 * signal runs its default action. Set the flag false to end that. */
int32_t lust_shim_signal_register_conditional_default(int32_t signal, uint64_t flag);

int32_t lust_shim_signals_new(const int32_t *signals, size_t count, uint64_t *out_handle);
int32_t lust_shim_signals_add(uint64_t handle, int32_t signal);
/* Blocks until at least one signal is pending, then writes the pending
 * batch (signal-hook wait). Returns out_n 0 when the handle is closed. */
int32_t lust_shim_signals_wait_batch(uint64_t handle, int32_t *out, size_t cap, size_t *out_n);
/* The pending batch without blocking (signal-hook pending). */
int32_t lust_shim_signals_pending_batch(uint64_t handle, int32_t *out, size_t cap, size_t *out_n);
/* Blocks for one signal (signal-hook forever next). out_has is 0 when
 * the handle is closed and the queue is empty. */
int32_t lust_shim_signals_next(uint64_t handle, int32_t *out_signal, int32_t *out_has);
/* Closes the queue from any thread. Blocked wait and next calls return
 * with no signal. The handle stays valid until signals_free. */
int32_t lust_shim_signals_close(uint64_t handle);
int32_t lust_shim_signals_free(uint64_t handle);
/* Sets the default action, unblocks the signal, and raises it
 * (signal-hook low_level emulate_default_handler). This can end the
 * process. It returns only when the default action does not. */
int32_t lust_shim_signal_emulate_default(int32_t signal);

/* ------------------------------------------------------------------ */
/* File watching (rustix inotify, nix fanotify, name-to-handle-at).   */
/* Event bytes stay in kernel layout: read the fanotify or inotify   */
/* fd with lust_shim_read and parse records on the caller side, as    */
/* ts-rust fswatch does.                                              */
/* ------------------------------------------------------------------ */

int32_t lust_shim_inotify_init1(int32_t flags, int32_t *out_fd);
int32_t lust_shim_inotify_add_watch(int32_t fd, const uint8_t *path, size_t path_len,
                                    uint32_t mask, int32_t *out_wd);
int32_t lust_shim_inotify_rm_watch(int32_t fd, int32_t wd);
int32_t lust_shim_fanotify_init(uint32_t flags, uint32_t event_f_flags, int32_t *out_fd);
/* has_path 0 passes no pathname, as Go passes nil for an empty path. */
int32_t lust_shim_fanotify_mark(int32_t fd, uint32_t flags, uint64_t mask,
                                int32_t dirfd, const uint8_t *path, size_t path_len,
                                int32_t has_path);
int32_t lust_shim_name_to_handle_at(int32_t dirfd, const uint8_t *path, size_t path_len,
                                    int32_t flags, int32_t *out_handle_type,
                                    int32_t *out_mount_id, LustShimBuf *out_handle);

/* ------------------------------------------------------------------ */
/* Data services: gzip (flate2), xxh3 (xxhash-rust), time (jiff),    */
/* big integers (num-bigint). Whole-buffer and whole-value calls,     */
/* which is the shape of every ts-rust call site for these crates.    */
/* ------------------------------------------------------------------ */

/* level is the flate2 level: 0 none, 1 fast (the pprof use), 6 the
 * flate2 default, 9 best. Output is one gzip member. */
int32_t lust_shim_gzip_encode(const uint8_t *in, size_t in_len, uint32_t level,
                              LustShimBuf *out);
/* Multi-member gzip decode (flate2 MultiGzDecoder), the locale use. */
int32_t lust_shim_gzip_decode_multi(const uint8_t *in, size_t in_len, LustShimBuf *out);

int32_t lust_shim_xxh3_64(const uint8_t *in, size_t in_len, uint64_t *out);
/* The u128 digest crosses as its low and high u64 words. */
int32_t lust_shim_xxh3_128(const uint8_t *in, size_t in_len,
                           uint64_t *out_lo, uint64_t *out_hi);
int32_t lust_shim_xxh3_new(uint64_t *out_handle);
int32_t lust_shim_xxh3_update(uint64_t handle, const uint8_t *in, size_t in_len);
int32_t lust_shim_xxh3_digest64(uint64_t handle, uint64_t *out);
int32_t lust_shim_xxh3_digest128(uint64_t handle, uint64_t *out_lo, uint64_t *out_hi);
int32_t lust_shim_xxh3_free(uint64_t handle);

typedef struct {
    int32_t year;
    int32_t month;
    int32_t day;
    int32_t hour;
    int32_t minute;
    int32_t second;
    int32_t subsec_nanos;
} LustShimDateTime;

/* A jiff time zone behind a handle. from_tzif takes the raw TZif file
 * bytes and the zone name (UTF-8, lossy, as ts-rust passes it). local
 * resolves the zone the way ts-rust execute/tsc/diagnostics.rs
 * init_local does on Unix (TZ env, then /etc/localtime and the zone
 * source dirs, else UTC). It reads the environment at call time, so
 * call it once and keep the handle. */
int32_t lust_shim_jiff_tz_utc(uint64_t *out_handle);
int32_t lust_shim_jiff_tz_from_tzif(const uint8_t *name, size_t name_len,
                                    const uint8_t *data, size_t data_len,
                                    uint64_t *out_handle);
int32_t lust_shim_jiff_tz_local(uint64_t *out_handle);
int32_t lust_shim_jiff_tz_free(uint64_t handle);
/* Civil time of a Unix timestamp in the zone. Falls with ERR when the
 * timestamp is outside the jiff range (years -9999 to 9999). */
int32_t lust_shim_jiff_tz_to_datetime(uint64_t tz, int64_t unix_sec, int32_t nanos,
                                      LustShimDateTime *out);
/* The two clocks ts-rust prints. kind 0: status clock "03:04:05 PM"
 * (execute/tsc/diagnostics.rs format_status_time). kind 1: log clock
 * "15:04:05.007" (project/logging/logger.rs format_clock_millis_in).
 * Output is ASCII in a shim buffer, no NUL byte at the end. */
#define LUST_SHIM_CLOCK_STATUS 0u
#define LUST_SHIM_CLOCK_MILLIS 1u
int32_t lust_shim_jiff_format_clock(uint64_t tz, int64_t unix_sec, int32_t nanos,
                                    uint32_t kind, LustShimBuf *out);

/* The three BigInt calls in ts-rust jsnum.rs, whole-value. out_has is
 * 0 when Rust would give None (a bad digit, or no f64 value). radix is
 * 2 to 36, as BigInt::parse_bytes takes. */
int32_t lust_shim_bigint_parse_to_f64(const uint8_t *digits, size_t digits_len,
                                      uint32_t radix, double *out, int32_t *out_has);
int32_t lust_shim_bigint_parse_to_decimal(const uint8_t *digits, size_t digits_len,
                                          uint32_t radix, LustShimBuf *out, int32_t *out_has);
int32_t lust_shim_bigint_pow_i64_to_f64(int64_t base, uint32_t power,
                                        double *out, int32_t *out_has);

/* ------------------------------------------------------------------ */
/* jemalloc control (tikv-jemalloc-ctl). In a build without the       */
/* jemalloc feature, enabled gives 0 and the rest return              */
/* LUST_SHIM_ERR_UNSUPPORTED. They are never silent no-ops in a       */
/* jemalloc build: thread arena assignment is real.                   */
/* ------------------------------------------------------------------ */

int32_t lust_shim_jemalloc_enabled(int32_t *out_enabled);
int32_t lust_shim_jemalloc_thread_arena_get(uint32_t *out_arena);
int32_t lust_shim_jemalloc_thread_arena_set(uint32_t arena);
int32_t lust_shim_jemalloc_opt_narenas(uint32_t *out_arenas);
/* The ts-rust jemalloc_layout memory layout: one 4 MiB allocation
 * with 4 MiB alignment in each arena, freed at once, plus one parked
 * thread named "arena-hold" bound to the last arena. Call at most
 * once, at process start. When to call it (env flags, memory limits)
 * stays caller policy, as in ts-rust jemalloc_layout.rs. */
int32_t lust_shim_jemalloc_layout(void);

#ifdef __cplusplus
}
#endif

#endif /* LUST_SHIM_H */
