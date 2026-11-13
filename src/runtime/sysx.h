/* The Unix calls behind the rustix adapter (w-cinterop2): the C
   behind the `sys_x_*` intrinsics of src/ty/lang.rs, called from
   library/deps/rustix. Paths cross as (ptr, len) byte strings and
   are copied to a NUL-terminated buffer here; an empty path with
   a real dirfd stays empty (AT_EMPTY_PATH callers pass the flag).
   OS errors return as errno (positive), or as -errno where the
   value is an fd or a flag word. Stat records are 8-byte
   little-endian fields appended to the out Vec (sysx.c lists
   them).

   Declarations here are in every unit; bodies (sysx.c) only in
   the runtime unit (LUST_IMPL), like sysnet.h. */
#ifndef LUST_RT_SYSX_H
#define LUST_RT_SYSX_H

/* kill(2): 0 or errno. */
int32_t lust_sys_x_kill(int32_t pid, int32_t sig);
/* getppid(2). */
int32_t lust_sys_x_getppid(void);
/* prctl(PR_SET_PDEATHSIG, sig): 0 or errno. */
int32_t lust_sys_x_prctl_deathsig(int32_t sig);
/* waitid(2) with the raw idtype, id and options: on success appends
   si_pid, si_uid, si_signo, si_status and si_code as five i64 (all 0
   when WNOHANG found no child); 0 or errno. */
int32_t lust_sys_x_waitid(int32_t idtype, int32_t id, int32_t options, lust_Vec *out);
/* pthread_sigmask(how, set, NULL) with `set` as a bit mask of the
   signals 1..64 (bit n-1 is signal n): 0 or errno. For nix. */
int32_t lust_sys_x_sigmask(int32_t how, uint64_t set);
/* prctl THP disable: set (mode 1) returns 0 or -errno; get
   (mode 0) returns 0 or 1, or -errno. */
int32_t lust_sys_x_prctl_thp(int32_t mode, int32_t val);
/* getrlimit: appends current and maximum as two u64 (RLIM_INFINITY
   is UINT64_MAX); 0 or errno. */
int32_t lust_sys_x_getrlimit(int32_t resource, lust_Vec *out);
/* setrlimit: 0 or errno. */
int32_t lust_sys_x_setrlimit(int32_t resource, uint64_t current, uint64_t maximum);
/* openat(2) with the raw Linux flags and mode: the fd or -errno. */
int32_t lust_sys_x_openat(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, uint32_t mode);
/* fstatat(2): appends the 16-field stat record; 0 or errno. */
int32_t lust_sys_x_statat(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, lust_Vec *out);
/* fstat(2): appends the 16-field stat record; 0 or errno. */
int32_t lust_sys_x_fstat(int32_t fd, lust_Vec *out);
/* statx(2): appends the 23-field statx record; 0 or errno. */
int32_t lust_sys_x_statx(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, uint32_t mask, lust_Vec *out);
/* faccessat2 (faccessat fallback without AT_EACCESS): 0 or errno. */
int32_t lust_sys_x_accessat(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t mode, int32_t flags);
/* utimensat(2) with both times as (sec, nsec): 0 or errno. */
int32_t lust_sys_x_utimensat(int32_t dirfd, const uint8_t *p, uint64_t len, int64_t a_sec, int64_t a_nsec, int64_t m_sec, int64_t m_nsec, int32_t flags);
/* mknodat(2): 0 or errno. */
int32_t lust_sys_x_mknodat(int32_t dirfd, const uint8_t *p, uint64_t len, uint32_t mode, uint64_t dev);
/* fcntl(F_GETFL): the flags or -errno. */
int32_t lust_sys_x_fcntl_getfl(int32_t fd);
/* fcntl(F_SETFL): 0 or errno. */
int32_t lust_sys_x_fcntl_setfl(int32_t fd, int32_t flags);
/* ioctl(TIOCGWINSZ): appends row, col, xpixel, ypixel as u16
   little-endian pairs (8 bytes); 0 or errno. */
int32_t lust_sys_x_winsize(int32_t fd, lust_Vec *out);
/* Signal registry behind the signal-hook adapter. Actions by
   kind: 0 set *flag to 1, 1 emulate the default action when
   *flag is set, 2 store value in the u64 at flag, 3 _exit(value)
   when *flag is set, 4 write the signal byte to a pipe fd.
   Register returns 0 or errno. */
/* Registers one action of a kind on a signal. */
int32_t lust_sys_sig_register(int32_t sig, int32_t kind, void *flag, uint64_t value);
/* Registers a pipe write end on a signal: 0 or errno. */
int32_t lust_sys_sig_register_pipe(int32_t sig, int32_t write_fd);
/* Removes every pipe action with this write end. */
void lust_sys_sig_unregister_pipe(int32_t write_fd);
/* pipe2 for a Signals instance: read end nonblocking, both
   CLOEXEC; appends the write fd as one u64, returns the read fd
   or -errno. */
int32_t lust_sys_sig_pipe(lust_Vec *out);
/* Waits for bytes on a Signals pipe, then drains what is there
   into `out`: the byte count, or -errno. */
int32_t lust_sys_sig_poll_read(int32_t fd, lust_Vec *out);
/* One nonblocking read of a Signals pipe into `out`: the byte
   count (0 when nothing waits), or -errno. */
int32_t lust_sys_sig_read_now(int32_t fd, lust_Vec *out);
/* raise(sig): 0 or errno. */
int32_t lust_sys_sig_raise(int32_t sig);
/* Restores SIG_DFL, unblocks, and raises; only returns (with an
   errno) when the restore itself fails, else aborts. */
int32_t lust_sys_sig_restore_raise(int32_t sig);
/* _exit(status). */
void lust_sys_sig_exit(int32_t status);
/* abort(). */
void lust_sys_sig_abort(void);
/* pipe2(2) with the raw Linux flags: appends the two fds as 8-byte
   fields; 0 or errno. */
int32_t lust_sys_x_pipe(int32_t flags, lust_Vec *out);
/* dup2(2): the new fd or -errno. */
int32_t lust_sys_x_dup2(int32_t oldfd, int32_t newfd);
/* poll(2) over `recs`: one 8-byte field per fd (fd in the low 32
   bits, events at bit 32, revents at bit 48), revents written
   back in place. Returns the ready count or -errno. A timeout
   of (sec, nsec) with sec<0 waits without a limit. */
int32_t lust_sys_x_poll(lust_Vec *recs, int64_t sec, int64_t nsec);
/* inotify_init1(2) with the raw Linux flags: the fd or -errno. */
int32_t lust_sys_x_inotify_init1(int32_t flags);
/* inotify_add_watch(2): the watch descriptor or -errno. */
int32_t lust_sys_x_inotify_add_watch(int32_t fd, const uint8_t *p, uint64_t len, uint32_t mask);
/* inotify_rm_watch(2): 0 or errno. */
int32_t lust_sys_x_inotify_rm_watch(int32_t fd, int32_t wd);
/* One getdents64 call of at most `max` bytes: the raw records
   are appended to `out`; the byte count or -errno. */
int64_t lust_sys_x_getdents(int32_t fd, uint64_t max, lust_Vec *out);
/* statvfs(2): appends the 11-field statvfs record (bsize,
   frsize, blocks, bfree, bavail, files, ffree, favail, fsid,
   flag, namemax) as u64s; 0 or errno. */
int32_t lust_sys_x_statvfs(const uint8_t *p, uint64_t len, lust_Vec *out);
/* fstatvfs(2): the same record for an fd; 0 or errno. */
int32_t lust_sys_x_fstatvfs(int32_t fd, lust_Vec *out);
/* fanotify_init(2) with the raw Linux flags (nix adapter):
   the fd or -errno. */
int32_t lust_sys_x_fanotify_init(uint32_t flags, uint32_t event_f_flags);
/* fanotify_mark(2): a path of length 0 with has_path 0 passes
   NULL as the pathname; 0 or errno. */
int32_t lust_sys_x_fanotify_mark(int32_t fd, uint32_t flags, uint64_t mask, int32_t dirfd, int32_t has_path, const uint8_t *p, uint64_t len);
/* name_to_handle_at(2) with the real crate's sizing loop:
   appends handle_type and the mount id (a full u64 field,
   for AT_HANDLE_MNT_ID_UNIQUE) and then the handle bytes;
   0 or errno. */
int32_t lust_sys_x_name_to_handle_at(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, lust_Vec *out);
/* open_by_handle_at(2) over a handle (type, bytes): the fd
   or -errno. */
int32_t lust_sys_x_open_by_handle_at(int32_t mount_fd, int32_t handle_type, const uint8_t *p, uint64_t len, int32_t flags);
/* kqueue(2) (the nix adapter's sys::event, Darwin): the fd or
   -errno. -ENOSYS off Darwin. */
int32_t lust_sys_x_kqueue(void);
/* kevent(2) over 48-byte records (ident u64, filter i64, flags
   u64, fflags u64, data i64, udata i64, all little-endian):
   `changes` is the changelist bytes, `events` a Vec whose length
   is the eventlist capacity in bytes and whose first records are
   overwritten with the returned events. has_timeout 0 waits
   without a limit. Returns the event count or -errno. */
int32_t lust_sys_x_kevent(int32_t fd, const uint8_t *changes, uint64_t clen, lust_Vec *events, int32_t has_timeout, int64_t sec, int64_t nsec);

#ifdef LUST_IMPL
#include "sysx.c"
#endif

#endif
