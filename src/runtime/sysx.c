#ifdef _WIN32
#include "sysxwin.c"
#else
/* The Unix calls behind the rustix adapter: see sysx.h. Included
   only under LUST_IMPL, after the runtime's own includes. The
   Linux-only calls (inotify, fanotify, prctl, statx, getdents64,
   the handle calls) compile only on Linux; off Linux their bodies
   return ENOSYS so every program still links, and the rustix
   adapter leaves those items out off Linux so a program that uses
   one rejects at compile time instead of meeting the stub. */
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <poll.h>
#ifdef __linux__
#include <sys/fanotify.h>
#include <sys/inotify.h>
#include <sys/prctl.h>
#include <sys/syscall.h>
#endif
#include <sys/ioctl.h>
#include <sys/statvfs.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <termios.h>
#include <unistd.h>

/* pipe2 with the caller's O_CLOEXEC/O_NONBLOCK flag bits (the
   target's values, from the rustix adapter): the call itself on
   Linux, pipe plus fcntl on Darwin, as in sysio.c. */
#ifdef __APPLE__
static int lust_sysx_pipe2(int fds[2], int flags) {
    if (pipe(fds) != 0) return -1;
    for (int i = 0; i < 2; i++) {
        if (flags & O_CLOEXEC) fcntl(fds[i], F_SETFD, FD_CLOEXEC);
        if (flags & O_NONBLOCK) fcntl(fds[i], F_SETFL, O_NONBLOCK);
    }
    return 0;
}
#else
static int lust_sysx_pipe2(int fds[2], int flags) { return pipe2(fds, flags); }
#endif

/* Appends `n` bytes to a Vec<u8> (the sysnet.c shape). */
static void lust_sysx_push(lust_Vec *out, const void *p, uint64_t n) {
    if (n == 0) return;
    lust_vec_reserve(out, n, 1, 1);
    memcpy((uint8_t *)out->ptr + out->len, p, (size_t)n);
    out->len += n;
}

/* Pushes one u64 to `out` little-endian. */
static void lust_sysx_u64(lust_Vec *out, uint64_t v) {
    uint8_t b[8];
    for (int k = 0; k < 8; k++) b[k] = (uint8_t)(v >> (8 * k));
    lust_sysx_push(out, b, 8);
}

/* A NUL-terminated copy of a (ptr, len) path, or NULL on OOM. */
static char *lust_sysx_path(const uint8_t *p, uint64_t len) {
    char *s = (char *)malloc(len + 1);
    if (!s) return NULL;
    memcpy(s, p, len);
    s[len] = 0;
    return s;
}

int32_t lust_sys_x_kill(int32_t pid, int32_t sig) {
    return kill((pid_t)pid, sig) == 0 ? 0 : errno;
}

int32_t lust_sys_x_getppid(void) {
    return (int32_t)getppid();
}

int32_t lust_sys_x_waitid(int32_t idtype, int32_t id, int32_t options, lust_Vec *out) {
    siginfo_t info;
    memset(&info, 0, sizeof info);
    if (waitid((idtype_t)idtype, (id_t)id, &info, options) != 0) return errno;
    lust_sysx_u64(out, (uint64_t)(int64_t)info.si_pid);
    lust_sysx_u64(out, (uint64_t)(int64_t)info.si_uid);
    lust_sysx_u64(out, (uint64_t)(int64_t)info.si_signo);
    lust_sysx_u64(out, (uint64_t)(int64_t)info.si_status);
    lust_sysx_u64(out, (uint64_t)(int64_t)info.si_code);
    return 0;
}

int32_t lust_sys_x_sigmask(int32_t how, uint64_t set) {
    sigset_t s;
    sigemptyset(&s);
    for (int n = 1; n <= 64; n++) {
        if (set >> (n - 1) & 1) sigaddset(&s, n);
    }
    return pthread_sigmask(how, &s, NULL);
}

int32_t lust_sys_x_prctl_deathsig(int32_t sig) {
#ifdef __linux__
    return prctl(PR_SET_PDEATHSIG, (unsigned long)sig, 0, 0, 0) == 0 ? 0 : errno;
#else
    (void)sig;
    return ENOSYS;
#endif
}

#ifndef PR_GET_THP_DISABLE
#define PR_GET_THP_DISABLE 41
#endif
#ifndef PR_SET_THP_DISABLE
#define PR_SET_THP_DISABLE 42
#endif

int32_t lust_sys_x_prctl_thp(int32_t mode, int32_t val) {
#ifdef __linux__
    if (mode == 1) {
        return prctl(PR_SET_THP_DISABLE, val != 0, 0, 0, 0) == 0 ? 0 : -errno;
    }
    int r = prctl(PR_GET_THP_DISABLE, 0, 0, 0, 0);
    return r < 0 ? -errno : r;
#else
    (void)mode;
    (void)val;
    return -ENOSYS;
#endif
}

int32_t lust_sys_x_getrlimit(int32_t resource, lust_Vec *out) {
    struct rlimit rl;
    if (getrlimit((int)resource, &rl) != 0) return errno;
    lust_sysx_u64(out, (uint64_t)rl.rlim_cur);
    lust_sysx_u64(out, (uint64_t)rl.rlim_max);
    return 0;
}

int32_t lust_sys_x_setrlimit(int32_t resource, uint64_t current, uint64_t maximum) {
    struct rlimit rl;
    rl.rlim_cur = (rlim_t)current;
    rl.rlim_max = (rlim_t)maximum;
    return setrlimit((int)resource, &rl) == 0 ? 0 : errno;
}

int32_t lust_sys_x_openat(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, uint32_t mode) {
    char *s = lust_sysx_path(p, len);
    if (!s) return -ENOMEM;
    int fd = openat(dirfd, s, flags, (mode_t)mode);
    int e = errno;
    free(s);
    return fd < 0 ? -e : fd;
}

/* The 16-field stat record: dev, ino, nlink, mode, uid, gid,
   rdev, size, blksize, blocks, atime sec/nsec, mtime sec/nsec,
   ctime sec/nsec (signed fields cross as their u64 bits). */
static void lust_sysx_stat_rec(lust_Vec *out, const struct stat *st) {
    uint64_t f[16] = {(uint64_t)st->st_dev, (uint64_t)st->st_ino, (uint64_t)st->st_nlink,
                      (uint64_t)st->st_mode, (uint64_t)st->st_uid, (uint64_t)st->st_gid,
                      (uint64_t)st->st_rdev, (uint64_t)st->st_size, (uint64_t)st->st_blksize,
                      (uint64_t)st->st_blocks, (uint64_t)LUST_ST_ATIME(*st).tv_sec, (uint64_t)LUST_ST_ATIME(*st).tv_nsec,
                      (uint64_t)LUST_ST_MTIME(*st).tv_sec, (uint64_t)LUST_ST_MTIME(*st).tv_nsec,
                      (uint64_t)LUST_ST_CTIME(*st).tv_sec, (uint64_t)LUST_ST_CTIME(*st).tv_nsec};
    for (int i = 0; i < 16; i++) lust_sysx_u64(out, f[i]);
}

int32_t lust_sys_x_statat(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, lust_Vec *out) {
    char *s = lust_sysx_path(p, len);
    if (!s) return ENOMEM;
    struct stat st;
    int r = fstatat(dirfd, s, &st, flags);
    int e = errno;
    free(s);
    if (r != 0) return e;
    lust_sysx_stat_rec(out, &st);
    return 0;
}

int32_t lust_sys_x_fstat(int32_t fd, lust_Vec *out) {
    struct stat st;
    if (fstat(fd, &st) != 0) return errno;
    lust_sysx_stat_rec(out, &st);
    return 0;
}

#ifdef __linux__
/* The kernel's struct statx (stable ABI, 256 bytes), spelled
   out because tcc's headers do not expose glibc's. Field order
   is the kernel's: atime, btime, ctime, mtime. */
struct lust_statx_time {
    int64_t tv_sec;
    uint32_t tv_nsec;
    int32_t reserved;
};
struct lust_statx {
    uint32_t stx_mask;
    uint32_t stx_blksize;
    uint64_t stx_attributes;
    uint32_t stx_nlink;
    uint32_t stx_uid;
    uint32_t stx_gid;
    uint16_t stx_mode;
    uint16_t spare0[1];
    uint64_t stx_ino;
    uint64_t stx_size;
    uint64_t stx_blocks;
    uint64_t stx_attributes_mask;
    struct lust_statx_time stx_atime;
    struct lust_statx_time stx_btime;
    struct lust_statx_time stx_ctime;
    struct lust_statx_time stx_mtime;
    uint32_t stx_rdev_major;
    uint32_t stx_rdev_minor;
    uint32_t stx_dev_major;
    uint32_t stx_dev_minor;
    uint64_t stx_mnt_id;
    uint32_t stx_dio_mem_align;
    uint32_t stx_dio_offset_align;
    uint64_t spare3[12];
};
#ifndef SYS_statx
#define SYS_statx 332
#endif

/* The statx record: mask, blksize, attributes, nlink, uid, gid,
   mode, ino, size, blocks, atime sec/nsec, mtime sec/nsec,
   ctime sec/nsec, btime sec/nsec, rdev major/minor, dev
   major/minor. */
int32_t lust_sys_x_statx(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, uint32_t mask, lust_Vec *out) {
    char *s = lust_sysx_path(p, len);
    if (!s) return ENOMEM;
    struct lust_statx stx;
    long r = syscall(SYS_statx, dirfd, s, flags, mask, &stx);
    int e = errno;
    free(s);
    if (r != 0) return e;
    uint64_t f[23] = {stx.stx_mask, stx.stx_blksize, stx.stx_attributes, stx.stx_nlink,
                      stx.stx_uid, stx.stx_gid, stx.stx_mode, stx.stx_ino,
                      stx.stx_size, stx.stx_blocks, stx.stx_attributes_mask, (uint64_t)stx.stx_atime.tv_sec, stx.stx_atime.tv_nsec,
                      (uint64_t)stx.stx_mtime.tv_sec, stx.stx_mtime.tv_nsec, (uint64_t)stx.stx_ctime.tv_sec,
                      stx.stx_ctime.tv_nsec, (uint64_t)stx.stx_btime.tv_sec, stx.stx_btime.tv_nsec,
                      stx.stx_rdev_major, stx.stx_rdev_minor, stx.stx_dev_major, stx.stx_dev_minor};
    for (int i = 0; i < 23; i++) lust_sysx_u64(out, f[i]);
    return 0;
}
#else
int32_t lust_sys_x_statx(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, uint32_t mask, lust_Vec *out) {
    (void)dirfd; (void)p; (void)len; (void)flags; (void)mask; (void)out;
    return ENOSYS;
}
#endif

int32_t lust_sys_x_accessat(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t mode, int32_t flags) {
    char *s = lust_sysx_path(p, len);
    if (!s) return ENOMEM;
#ifdef SYS_faccessat2
    long r = syscall(SYS_faccessat2, dirfd, s, mode, flags);
    int e = errno;
    if (r != 0 && e == ENOSYS) {
        r = faccessat(dirfd, s, mode, flags & AT_EACCESS);
        e = errno;
    }
#else
    long r = faccessat(dirfd, s, mode, flags & AT_EACCESS);
    int e = errno;
#endif
    free(s);
    return r == 0 ? 0 : e;
}

int32_t lust_sys_x_utimensat(int32_t dirfd, const uint8_t *p, uint64_t len, int64_t a_sec, int64_t a_nsec, int64_t m_sec, int64_t m_nsec, int32_t flags) {
    char *s = lust_sysx_path(p, len);
    if (!s) return ENOMEM;
    struct timespec ts[2] = {{(time_t)a_sec, (long)a_nsec}, {(time_t)m_sec, (long)m_nsec}};
    int r = utimensat(dirfd, s, ts, flags);
    int e = errno;
    free(s);
    return r == 0 ? 0 : e;
}

int32_t lust_sys_x_mknodat(int32_t dirfd, const uint8_t *p, uint64_t len, uint32_t mode, uint64_t dev) {
    char *s = lust_sysx_path(p, len);
    if (!s) return ENOMEM;
    int r = mknodat(dirfd, s, (mode_t)mode, (dev_t)dev);
    int e = errno;
    free(s);
    return r == 0 ? 0 : e;
}

int32_t lust_sys_x_fcntl_getfl(int32_t fd) {
    int r = fcntl(fd, F_GETFL);
    return r < 0 ? -errno : r;
}

int32_t lust_sys_x_fcntl_setfl(int32_t fd, int32_t flags) {
    return fcntl(fd, F_SETFL, flags) == 0 ? 0 : errno;
}

int32_t lust_sys_x_pipe(int32_t flags, lust_Vec *out) {
    int fds[2];
    if (lust_sysx_pipe2(fds, flags) != 0) return errno;
    lust_sysx_u64(out, (uint64_t)(uint32_t)fds[0]);
    lust_sysx_u64(out, (uint64_t)(uint32_t)fds[1]);
    return 0;
}

int32_t lust_sys_x_dup2(int32_t oldfd, int32_t newfd) {
    int r = dup2(oldfd, newfd);
    return r < 0 ? -errno : r;
}

int32_t lust_sys_x_poll(lust_Vec *recs, int64_t sec, int64_t nsec) {
    uint64_t n = recs->len / 8;
    struct pollfd stack_fds[16];
    struct pollfd *fds = stack_fds;
    if (n > 16) {
        fds = (struct pollfd *)malloc((size_t)n * sizeof(struct pollfd));
        if (!fds) return -ENOMEM;
    }
    const uint8_t *bytes = (const uint8_t *)recs->ptr;
    for (uint64_t i = 0; i < n; i++) {
        uint64_t v = 0;
        for (int k = 0; k < 8; k++) v |= (uint64_t)bytes[i * 8 + k] << (8 * k);
        fds[i].fd = (int)(uint32_t)v;
        fds[i].events = (short)(v >> 32);
        fds[i].revents = 0;
    }
    int timeout_ms = -1;
    if (sec >= 0) {
        int64_t ms = sec * 1000 + nsec / 1000000 + (nsec % 1000000 != 0 ? 1 : 0);
        timeout_ms = ms > INT32_MAX ? INT32_MAX : (int)ms;
    }
    int r = poll(fds, (nfds_t)n, timeout_ms);
    int e = errno;
    if (r >= 0) {
        uint8_t *out = (uint8_t *)recs->ptr;
        for (uint64_t i = 0; i < n; i++) {
            uint16_t rev = (uint16_t)fds[i].revents;
            out[i * 8 + 6] = (uint8_t)(rev & 0xff);
            out[i * 8 + 7] = (uint8_t)(rev >> 8);
        }
    }
    if (fds != stack_fds) free(fds);
    return r < 0 ? -e : r;
}

int32_t lust_sys_x_winsize(int32_t fd, lust_Vec *out) {
    struct winsize ws;
    if (ioctl(fd, TIOCGWINSZ, &ws) != 0) return errno;
    uint16_t f[4] = {ws.ws_row, ws.ws_col, ws.ws_xpixel, ws.ws_ypixel};
    uint8_t b[8];
    for (int i = 0; i < 4; i++) {
        b[2 * i] = (uint8_t)(f[i] & 0xff);
        b[2 * i + 1] = (uint8_t)(f[i] >> 8);
    }
    lust_sysx_push(out, b, 8);
    return 0;
}

/* ---- Signal registry (signal-hook adapter) ---- */

#define LUST_SIG_MAX 64
#define LUST_SIG_ACTIONS 24

struct lust_sig_action {
    int32_t kind;
    volatile uint8_t *flag;
    uint64_t value;
    int32_t pipe_fd;
};

static struct lust_sig_action lust_sig_actions[LUST_SIG_MAX + 1][LUST_SIG_ACTIONS];
static volatile sig_atomic_t lust_sig_counts[LUST_SIG_MAX + 1];
static int lust_sig_installed[LUST_SIG_MAX + 1];

/* The Term case of emulate_default_handler, shared with the
   conditional-default action. Async-signal-safe calls only. */
static void lust_sig_emulate_term(int sig) {
    struct sigaction dfl;
    memset(&dfl, 0, sizeof dfl);
    dfl.sa_handler = SIG_DFL;
    sigaction(sig, &dfl, NULL);
    sigset_t set;
    sigemptyset(&set);
    sigaddset(&set, sig);
    pthread_sigmask(SIG_UNBLOCK, &set, NULL);
    raise(sig);
    abort();
}

/* The default-action kind of a signal, upstream's table:
   0 ignore, 1 stop, 2 terminate, -1 unknown. */
static int lust_sig_default_kind(int sig) {
    switch (sig) {
        case SIGCHLD:
        case SIGCONT:
        case SIGURG:
        case SIGWINCH:
        case SIGIO:
            return 0;
        case SIGSTOP:
        case SIGTSTP:
        case SIGTTIN:
        case SIGTTOU:
            return 1;
        case SIGHUP:
        case SIGINT:
        case SIGQUIT:
        case SIGILL:
        case SIGTRAP:
        case SIGABRT:
        case SIGBUS:
        case SIGFPE:
        case SIGKILL:
        case SIGUSR1:
        case SIGSEGV:
        case SIGUSR2:
        case SIGPIPE:
        case SIGALRM:
        case SIGTERM:
        case SIGPROF:
        case SIGSYS:
        case SIGVTALRM:
        case SIGXCPU:
        case SIGXFSZ:
            return 2;
        default:
            return -1;
    }
}

static void lust_sig_handler(int sig) {
    int saved = errno;
    if (sig >= 1 && sig <= LUST_SIG_MAX) {
        int n = (int)lust_sig_counts[sig];
        for (int i = 0; i < n; i++) {
            struct lust_sig_action *a = &lust_sig_actions[sig][i];
            switch (a->kind) {
                case 0:
                    *a->flag = 1;
                    break;
                case 1:
                    if (*a->flag) {
                        int kind = lust_sig_default_kind(sig);
                        if (kind == 2) lust_sig_emulate_term(sig);
                        else if (kind == 1) raise(SIGSTOP);
                    }
                    break;
                case 2:
                    *(volatile uint64_t *)a->flag = a->value;
                    break;
                case 3:
                    if (*a->flag) _exit((int)a->value);
                    break;
                case 4: {
                    uint8_t b = (uint8_t)sig;
                    ssize_t unused = write(a->pipe_fd, &b, 1);
                    (void)unused;
                    break;
                }
                default:
                    break;
            }
        }
    }
    errno = saved;
}

/* Installs the shared handler for one signal, once. */
static int lust_sig_install(int sig) {
    if (lust_sig_installed[sig]) return 0;
    struct sigaction sa;
    memset(&sa, 0, sizeof sa);
    sa.sa_handler = lust_sig_handler;
    sa.sa_flags = SA_RESTART;
    sigemptyset(&sa.sa_mask);
    if (sigaction(sig, &sa, NULL) != 0) return errno;
    lust_sig_installed[sig] = 1;
    return 0;
}

int32_t lust_sys_sig_register(int32_t sig, int32_t kind, void *flag, uint64_t value) {
    if (sig < 1 || sig > LUST_SIG_MAX) return EINVAL;
    int n = (int)lust_sig_counts[sig];
    if (n >= LUST_SIG_ACTIONS) return ENOMEM;
    int e = lust_sig_install(sig);
    if (e) return e;
    lust_sig_actions[sig][n].kind = kind;
    lust_sig_actions[sig][n].flag = (volatile uint8_t *)flag;
    lust_sig_actions[sig][n].value = value;
    lust_sig_actions[sig][n].pipe_fd = -1;
    lust_sig_counts[sig] = n + 1;
    return 0;
}

int32_t lust_sys_sig_register_pipe(int32_t sig, int32_t write_fd) {
    if (sig < 1 || sig > LUST_SIG_MAX) return EINVAL;
    int n = (int)lust_sig_counts[sig];
    if (n >= LUST_SIG_ACTIONS) return ENOMEM;
    int e = lust_sig_install(sig);
    if (e) return e;
    lust_sig_actions[sig][n].kind = 4;
    lust_sig_actions[sig][n].flag = NULL;
    lust_sig_actions[sig][n].value = 0;
    lust_sig_actions[sig][n].pipe_fd = write_fd;
    lust_sig_counts[sig] = n + 1;
    return 0;
}

void lust_sys_sig_unregister_pipe(int32_t write_fd) {
    for (int sig = 1; sig <= LUST_SIG_MAX; sig++) {
        int n = (int)lust_sig_counts[sig];
        int w = 0;
        for (int i = 0; i < n; i++) {
            if (lust_sig_actions[sig][i].kind == 4 && lust_sig_actions[sig][i].pipe_fd == write_fd) continue;
            lust_sig_actions[sig][w] = lust_sig_actions[sig][i];
            w++;
        }
        lust_sig_counts[sig] = w;
    }
}

int32_t lust_sys_sig_pipe(lust_Vec *out) {
    int fds[2];
    if (lust_sysx_pipe2(fds, O_CLOEXEC | O_NONBLOCK) != 0) return -errno;
    lust_sysx_u64(out, (uint64_t)(uint32_t)fds[1]);
    return fds[0];
}

int32_t lust_sys_sig_poll_read(int32_t fd, lust_Vec *out) {
    struct pollfd pfd;
    pfd.fd = fd;
    pfd.events = POLLIN;
    pfd.revents = 0;
    int r;
    do {
        r = poll(&pfd, 1, -1);
    } while (r < 0 && errno == EINTR);
    if (r < 0) return -errno;
    uint8_t buf[128];
    ssize_t n = read(fd, buf, sizeof buf);
    if (n < 0) return errno == EAGAIN ? 0 : -errno;
    lust_sysx_push(out, buf, (uint64_t)n);
    return (int32_t)n;
}

int32_t lust_sys_sig_read_now(int32_t fd, lust_Vec *out) {
    uint8_t buf[128];
    ssize_t n = read(fd, buf, sizeof buf);
    if (n < 0) return errno == EAGAIN ? 0 : -errno;
    lust_sysx_push(out, buf, (uint64_t)n);
    return (int32_t)n;
}

int32_t lust_sys_sig_raise(int32_t sig) {
    return raise(sig) == 0 ? 0 : errno;
}

int32_t lust_sys_sig_restore_raise(int32_t sig) {
    struct sigaction dfl;
    memset(&dfl, 0, sizeof dfl);
    dfl.sa_handler = SIG_DFL;
    if (sigaction(sig, &dfl, NULL) != 0) return errno;
    lust_sig_installed[sig] = 0;
    lust_sig_counts[sig] = 0;
    sigset_t set;
    sigemptyset(&set);
    sigaddset(&set, sig);
    pthread_sigmask(SIG_UNBLOCK, &set, NULL);
    raise(sig);
    abort();
    return 0;
}

void lust_sys_sig_exit(int32_t status) {
    _exit(status);
}

void lust_sys_sig_abort(void) {
    abort();
}

#ifdef __linux__
int32_t lust_sys_x_inotify_init1(int32_t flags) {
    int fd = inotify_init1(flags);
    return fd < 0 ? -errno : fd;
}

int32_t lust_sys_x_inotify_add_watch(int32_t fd, const uint8_t *p, uint64_t len, uint32_t mask) {
    char *s = lust_sysx_path(p, len);
    if (!s) return -ENOMEM;
    int wd = inotify_add_watch(fd, s, mask);
    int e = errno;
    free(s);
    return wd < 0 ? -e : wd;
}

int32_t lust_sys_x_inotify_rm_watch(int32_t fd, int32_t wd) {
    return inotify_rm_watch(fd, wd) == 0 ? 0 : errno;
}
#else
int32_t lust_sys_x_inotify_init1(int32_t flags) {
    (void)flags;
    return -ENOSYS;
}

int32_t lust_sys_x_inotify_add_watch(int32_t fd, const uint8_t *p, uint64_t len, uint32_t mask) {
    (void)fd; (void)p; (void)len; (void)mask;
    return -ENOSYS;
}

int32_t lust_sys_x_inotify_rm_watch(int32_t fd, int32_t wd) {
    (void)fd; (void)wd;
    return ENOSYS;
}
#endif

#ifdef __linux__
int64_t lust_sys_x_getdents(int32_t fd, uint64_t max, lust_Vec *out) {
    if (max == 0) {
        /* The kernel answers EINVAL for a zero-size buffer. */
        long r = syscall(SYS_getdents64, fd, (void *)0, (size_t)0);
        return r < 0 ? -errno : (int64_t)r;
    }
    uint8_t *buf = (uint8_t *)malloc((size_t)max);
    if (!buf) return -ENOMEM;
    long r = syscall(SYS_getdents64, fd, buf, (size_t)max);
    int e = errno;
    if (r > 0) lust_sysx_push(out, buf, (uint64_t)r);
    free(buf);
    return r < 0 ? -e : (int64_t)r;
}
#else
int64_t lust_sys_x_getdents(int32_t fd, uint64_t max, lust_Vec *out) {
    (void)fd; (void)max; (void)out;
    return -ENOSYS;
}
#endif

/* The 11-field statvfs record: bsize, frsize, blocks, bfree,
   bavail, files, ffree, favail, fsid, flag, namemax. */
static void lust_sysx_statvfs_rec(lust_Vec *out, const struct statvfs *st) {
    uint64_t f[11] = {(uint64_t)st->f_bsize, (uint64_t)st->f_frsize, (uint64_t)st->f_blocks,
                      (uint64_t)st->f_bfree, (uint64_t)st->f_bavail, (uint64_t)st->f_files,
                      (uint64_t)st->f_ffree, (uint64_t)st->f_favail, (uint64_t)st->f_fsid,
                      (uint64_t)st->f_flag, (uint64_t)st->f_namemax};
    for (int i = 0; i < 11; i++) lust_sysx_u64(out, f[i]);
}

int32_t lust_sys_x_statvfs(const uint8_t *p, uint64_t len, lust_Vec *out) {
    char *s = lust_sysx_path(p, len);
    if (!s) return ENOMEM;
    struct statvfs st;
    int r = statvfs(s, &st);
    int e = errno;
    free(s);
    if (r != 0) return e;
    lust_sysx_statvfs_rec(out, &st);
    return 0;
}

int32_t lust_sys_x_fstatvfs(int32_t fd, lust_Vec *out) {
    struct statvfs st;
    if (fstatvfs(fd, &st) != 0) return errno;
    lust_sysx_statvfs_rec(out, &st);
    return 0;
}

#ifdef __linux__
int32_t lust_sys_x_fanotify_init(uint32_t flags, uint32_t event_f_flags) {
    int fd = fanotify_init(flags, event_f_flags);
    return fd < 0 ? -errno : fd;
}

int32_t lust_sys_x_fanotify_mark(int32_t fd, uint32_t flags, uint64_t mask, int32_t dirfd, int32_t has_path, const uint8_t *p, uint64_t len) {
    char *s = NULL;
    if (has_path) {
        s = lust_sysx_path(p, len);
        if (!s) return ENOMEM;
    }
    int r = fanotify_mark(fd, flags, mask, dirfd, s);
    int e = errno;
    free(s);
    return r != 0 ? e : 0;
}
#else
int32_t lust_sys_x_fanotify_init(uint32_t flags, uint32_t event_f_flags) {
    (void)flags; (void)event_f_flags;
    return -ENOSYS;
}

int32_t lust_sys_x_fanotify_mark(int32_t fd, uint32_t flags, uint64_t mask, int32_t dirfd, int32_t has_path, const uint8_t *p, uint64_t len) {
    (void)fd; (void)flags; (void)mask; (void)dirfd; (void)has_path; (void)p; (void)len;
    return ENOSYS;
}
#endif

#ifdef __linux__
/* struct file_handle with a `cap`-byte handle tail. */
struct lust_sysx_fhandle {
    uint32_t handle_bytes;
    int32_t handle_type;
    uint8_t f_handle[];
};

int32_t lust_sys_x_name_to_handle_at(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, lust_Vec *out) {
    char *s = lust_sysx_path(p, len);
    if (!s) return ENOMEM;
    struct lust_sysx_fhandle probe;
    probe.handle_bytes = 0;
    probe.handle_type = 0;
    uint64_t mount_id = 0;
    long r = syscall(SYS_name_to_handle_at, dirfd, s, &probe, &mount_id, flags);
    int e = errno;
    if (r == 0 || e != EOVERFLOW || probe.handle_bytes == 0) {
        free(s);
        return r == 0 ? EOVERFLOW : e;
    }
    uint32_t want = probe.handle_bytes;
    int32_t handle_type = probe.handle_type;
    for (;;) {
        struct lust_sysx_fhandle *h = (struct lust_sysx_fhandle *)calloc(1, sizeof(*h) + want);
        if (!h) {
            free(s);
            return ENOMEM;
        }
        h->handle_bytes = want;
        h->handle_type = handle_type;
        r = syscall(SYS_name_to_handle_at, dirfd, s, h, &mount_id, flags);
        e = errno;
        uint32_t got = h->handle_bytes;
        handle_type = h->handle_type;
        if (r == 0) {
            lust_sysx_u64(out, (uint64_t)(uint32_t)handle_type);
            lust_sysx_u64(out, mount_id);
            lust_sysx_push(out, h->f_handle, got);
            free(h);
            free(s);
            return 0;
        }
        free(h);
        if (e != EOVERFLOW) {
            free(s);
            return e;
        }
        want = got;
    }
}

int32_t lust_sys_x_open_by_handle_at(int32_t mount_fd, int32_t handle_type, const uint8_t *p, uint64_t len, int32_t flags) {
    struct lust_sysx_fhandle *h = (struct lust_sysx_fhandle *)calloc(1, sizeof(*h) + len);
    if (!h) return -ENOMEM;
    h->handle_bytes = (uint32_t)len;
    h->handle_type = handle_type;
    memcpy(h->f_handle, p, len);
    long r = syscall(SYS_open_by_handle_at, mount_fd, h, flags);
    int e = errno;
    free(h);
    return r < 0 ? -e : (int32_t)r;
}
#else
int32_t lust_sys_x_name_to_handle_at(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, lust_Vec *out) {
    (void)dirfd; (void)p; (void)len; (void)flags; (void)out;
    return ENOSYS;
}

int32_t lust_sys_x_open_by_handle_at(int32_t mount_fd, int32_t handle_type, const uint8_t *p, uint64_t len, int32_t flags) {
    (void)mount_fd; (void)handle_type; (void)p; (void)len; (void)flags;
    return -ENOSYS;
}

#endif
#endif

/* kqueue/kevent for the nix adapter's sys::event (Darwin only; the
   adapter's event module exists only there, as upstream's). Records
   cross as 48-byte little-endian fields (sysx.h), translated to and
   from struct kevent here so its layout stays the C compiler's. */
#ifdef __APPLE__
#include <sys/event.h>
#include <sys/time.h>

static uint64_t lust_sysx_rec_u64(const uint8_t *p) {
    uint64_t v = 0;
    for (int k = 0; k < 8; k++) v |= (uint64_t)p[k] << (8 * k);
    return v;
}

static void lust_sysx_rec_put(uint8_t *p, const struct kevent *ev) {
    uint64_t f[6] = {(uint64_t)ev->ident, (uint64_t)(int64_t)ev->filter, ev->flags, ev->fflags,
                     (uint64_t)(int64_t)ev->data, (uint64_t)(int64_t)(uintptr_t)ev->udata};
    for (int i = 0; i < 6; i++)
        for (int k = 0; k < 8; k++) p[i * 8 + k] = (uint8_t)(f[i] >> (8 * k));
}

int32_t lust_sys_x_kqueue(void) {
    int fd = kqueue();
    return fd < 0 ? -errno : fd;
}

int32_t lust_sys_x_kevent(int32_t fd, const uint8_t *changes, uint64_t clen, lust_Vec *events, int32_t has_timeout, int64_t sec, int64_t nsec) {
    if (clen % 48 != 0 || events->len % 48 != 0) return -EINVAL;
    uint64_t nch = clen / 48, nev = events->len / 48;
    struct kevent stack_ch[8], stack_ev[8];
    struct kevent *ch = stack_ch, *ev = stack_ev;
    if (nch > 8) {
        ch = (struct kevent *)malloc((size_t)nch * sizeof *ch);
        if (!ch) return -ENOMEM;
    }
    if (nev > 8) {
        ev = (struct kevent *)malloc((size_t)nev * sizeof *ev);
        if (!ev) {
            if (ch != stack_ch) free(ch);
            return -ENOMEM;
        }
    }
    for (uint64_t i = 0; i < nch; i++) {
        const uint8_t *r = changes + i * 48;
        memset(&ch[i], 0, sizeof ch[i]);
        ch[i].ident = (uintptr_t)lust_sysx_rec_u64(r);
        ch[i].filter = (int16_t)(int64_t)lust_sysx_rec_u64(r + 8);
        ch[i].flags = (uint16_t)lust_sysx_rec_u64(r + 16);
        ch[i].fflags = (uint32_t)lust_sysx_rec_u64(r + 24);
        ch[i].data = (intptr_t)(int64_t)lust_sysx_rec_u64(r + 32);
        ch[i].udata = (void *)(uintptr_t)lust_sysx_rec_u64(r + 40);
    }
    struct timespec ts;
    ts.tv_sec = (time_t)sec;
    ts.tv_nsec = (long)nsec;
    int n = kevent(fd, ch, (int)nch, ev, (int)nev, has_timeout ? &ts : NULL);
    int e = errno;
    if (n > 0) {
        uint8_t *out = (uint8_t *)events->ptr;
        for (int i = 0; i < n; i++) lust_sysx_rec_put(out + (uint64_t)i * 48, &ev[i]);
    }
    if (ch != stack_ch) free(ch);
    if (ev != stack_ev) free(ev);
    return n < 0 ? -e : n;
}
#else
int32_t lust_sys_x_kqueue(void) { return -ENOSYS; }

int32_t lust_sys_x_kevent(int32_t fd, const uint8_t *changes, uint64_t clen, lust_Vec *events, int32_t has_timeout, int64_t sec, int64_t nsec) {
    (void)fd; (void)changes; (void)clen; (void)events; (void)has_timeout; (void)sec; (void)nsec;
    return -ENOSYS;
}
#endif
