/* Linux-only intrinsics (rustix, signal-hook, nix adapters) for
   Windows (rel-win). Spliced instead of sysx.c under _WIN32.
   Real here: pipes (CreatePipe), console size, exit and abort.
   The rest fail with ERROR_NOT_SUPPORTED (50): their crates
   (rustix, signal-hook) keep tsgo's Windows build a reject at
   the Rust level until Windows adapters land; see REPORT.md. */
#define LUST_X_NOSUP 50

static void lust_sysx_push_u64(lust_Vec *out, uint64_t v) {
    lust_vec_reserve(out, 8, 1, 1);
    uint8_t *b = (uint8_t *)out->ptr + out->len;
    for (int k = 0; k < 8; k++) b[k] = (uint8_t)(v >> (8 * k));
    out->len += 8;
}

int32_t lust_sys_x_pipe(int32_t flags, lust_Vec *out) {
    (void)flags;
    HANDLE rd = INVALID_HANDLE_VALUE, wr = INVALID_HANDLE_VALUE;
    if (!CreatePipe(&rd, &wr, 0, 0)) return (int32_t)GetLastError();
    lust_sysx_push_u64(out, (uint64_t)lust_win_fd_new(rd, 0));
    lust_sysx_push_u64(out, (uint64_t)lust_win_fd_new(wr, 0));
    return 0;
}

int32_t lust_sys_x_winsize(int32_t fd, lust_Vec *out) {
    HANDLE h = lust_win_handle(fd);
    CONSOLE_SCREEN_BUFFER_INFO info;
    if (h == INVALID_HANDLE_VALUE || !GetConsoleScreenBufferInfo(h, &info)) return (int32_t)GetLastError();
    lust_sysx_push_u64(out, (uint64_t)(info.srWindow.Bottom - info.srWindow.Top + 1));
    lust_sysx_push_u64(out, (uint64_t)(info.srWindow.Right - info.srWindow.Left + 1));
    lust_sysx_push_u64(out, 0);
    lust_sysx_push_u64(out, 0);
    return 0;
}

void lust_sys_sig_exit(int32_t status) { ExitProcess((UINT)status); }
void lust_sys_sig_abort(void) { abort(); }

int32_t lust_sys_x_kill(int32_t pid, int32_t sig) { (void)pid; (void)sig; return LUST_X_NOSUP; }
int32_t lust_sys_x_prctl_deathsig(int32_t sig) { (void)sig; return LUST_X_NOSUP; }
int32_t lust_sys_x_waitid(int32_t idtype, int32_t id, int32_t options, lust_Vec *out) { (void)idtype; (void)id; (void)options; (void)out; return LUST_X_NOSUP; }
int32_t lust_sys_x_sigmask(int32_t how, uint64_t set) { (void)how; (void)set; return LUST_X_NOSUP; }
int32_t lust_sys_x_prctl_thp(int32_t mode, int32_t val) { (void)mode; (void)val; return LUST_X_NOSUP; }
int32_t lust_sys_x_getrlimit(int32_t resource, lust_Vec *out) { (void)resource; (void)out; return LUST_X_NOSUP; }
int32_t lust_sys_x_setrlimit(int32_t resource, uint64_t current, uint64_t maximum) { (void)resource; (void)current; (void)maximum; return LUST_X_NOSUP; }
int32_t lust_sys_x_openat(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, uint32_t mode) { (void)dirfd; (void)p; (void)len; (void)flags; (void)mode; return LUST_X_NOSUP; }
int32_t lust_sys_x_statat(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, lust_Vec *out) { (void)dirfd; (void)p; (void)len; (void)flags; (void)out; return LUST_X_NOSUP; }
int32_t lust_sys_x_fstat(int32_t fd, lust_Vec *out) { (void)fd; (void)out; return LUST_X_NOSUP; }
int32_t lust_sys_x_statx(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, uint32_t mask, lust_Vec *out) { (void)dirfd; (void)p; (void)len; (void)flags; (void)mask; (void)out; return LUST_X_NOSUP; }
int32_t lust_sys_x_accessat(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t mode, int32_t flags) { (void)dirfd; (void)p; (void)len; (void)mode; (void)flags; return LUST_X_NOSUP; }
int32_t lust_sys_x_utimensat(int32_t dirfd, const uint8_t *p, uint64_t len, int64_t a_sec, int64_t a_nsec, int64_t m_sec, int64_t m_nsec, int32_t flags) { (void)dirfd; (void)p; (void)len; (void)a_sec; (void)a_nsec; (void)m_sec; (void)m_nsec; (void)flags; return LUST_X_NOSUP; }
int32_t lust_sys_x_mknodat(int32_t dirfd, const uint8_t *p, uint64_t len, uint32_t mode, uint64_t dev) { (void)dirfd; (void)p; (void)len; (void)mode; (void)dev; return LUST_X_NOSUP; }
int32_t lust_sys_x_fcntl_getfl(int32_t fd) { (void)fd; return LUST_X_NOSUP; }
int32_t lust_sys_x_fcntl_setfl(int32_t fd, int32_t flags) { (void)fd; (void)flags; return LUST_X_NOSUP; }
int32_t lust_sys_sig_register(int32_t sig, int32_t kind, void *flag, uint64_t value) { (void)sig; (void)kind; (void)flag; (void)value; return LUST_X_NOSUP; }
int32_t lust_sys_sig_register_pipe(int32_t sig, int32_t write_fd) { (void)sig; (void)write_fd; return LUST_X_NOSUP; }
int32_t lust_sys_sig_pipe(lust_Vec *out) { (void)out; return LUST_X_NOSUP; }
int32_t lust_sys_sig_poll_read(int32_t fd, lust_Vec *out) { (void)fd; (void)out; return LUST_X_NOSUP; }
int32_t lust_sys_sig_read_now(int32_t fd, lust_Vec *out) { (void)fd; (void)out; return LUST_X_NOSUP; }
int32_t lust_sys_sig_raise(int32_t sig) { (void)sig; return LUST_X_NOSUP; }
int32_t lust_sys_sig_restore_raise(int32_t sig) { (void)sig; return LUST_X_NOSUP; }
int32_t lust_sys_x_dup2(int32_t oldfd, int32_t newfd) { (void)oldfd; (void)newfd; return LUST_X_NOSUP; }
int32_t lust_sys_x_poll(lust_Vec *recs, int64_t sec, int64_t nsec) { (void)recs; (void)sec; (void)nsec; return LUST_X_NOSUP; }
int32_t lust_sys_x_inotify_init1(int32_t flags) { (void)flags; return LUST_X_NOSUP; }
int32_t lust_sys_x_inotify_add_watch(int32_t fd, const uint8_t *p, uint64_t len, uint32_t mask) { (void)fd; (void)p; (void)len; (void)mask; return LUST_X_NOSUP; }
int32_t lust_sys_x_inotify_rm_watch(int32_t fd, int32_t wd) { (void)fd; (void)wd; return LUST_X_NOSUP; }
int32_t lust_sys_x_statvfs(const uint8_t *p, uint64_t len, lust_Vec *out) { (void)p; (void)len; (void)out; return LUST_X_NOSUP; }
int32_t lust_sys_x_fstatvfs(int32_t fd, lust_Vec *out) { (void)fd; (void)out; return LUST_X_NOSUP; }
int32_t lust_sys_x_fanotify_init(uint32_t flags, uint32_t event_f_flags) { (void)flags; (void)event_f_flags; return LUST_X_NOSUP; }
int32_t lust_sys_x_fanotify_mark(int32_t fd, uint32_t flags, uint64_t mask, int32_t dirfd, int32_t has_path, const uint8_t *p, uint64_t len) { (void)fd; (void)flags; (void)mask; (void)dirfd; (void)has_path; (void)p; (void)len; return LUST_X_NOSUP; }
int32_t lust_sys_x_name_to_handle_at(int32_t dirfd, const uint8_t *p, uint64_t len, int32_t flags, lust_Vec *out) { (void)dirfd; (void)p; (void)len; (void)flags; (void)out; return LUST_X_NOSUP; }
int32_t lust_sys_x_open_by_handle_at(int32_t mount_fd, int32_t handle_type, const uint8_t *p, uint64_t len, int32_t flags) { (void)mount_fd; (void)handle_type; (void)p; (void)len; (void)flags; return LUST_X_NOSUP; }
int32_t lust_sys_x_getppid(void) { return 0; }
int64_t lust_sys_x_getdents(int32_t fd, uint64_t max, lust_Vec *out) { (void)fd; (void)max; (void)out; return -LUST_X_NOSUP; }
void lust_sys_sig_unregister_pipe(int32_t write_fd) { (void)write_fd; }
