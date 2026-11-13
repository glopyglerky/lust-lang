//! Go `runtime.GOMAXPROCS(0)`: the number of threads that run Go code at
//! once. The port sizes its thread pools with it (`gomaxprocs`).
//!
//! Go: runtime/proc.go:930 schedinit, runtime/cgroup_linux.go and
//! internal/runtime/cgroup (go1.27.1, the toolchain of tsc/go.mod `go
//! 1.27`, so the cgroup limit applies).
//! PORT: Go reads the CPUs and the cgroup limit again while it runs
//! (`updatemaxprocs`, sysmon). The port reads them once.
//! PORT: the port has no garbage collector, so `GOGC` and `GOMEMLIMIT` do
//! nothing (PORTING.md "Process start").

use std::sync::OnceLock;


// Go: cgroup_linux.go:85 defaultGOMAXPROCS and :109 adjustCgroupGOMAXPROCS

// Go: os_linux.go:101 getCPUCount: the CPUs in the sched_getaffinity mask,
// at least 1.
// PORT: the mask is `Cpus_allowed` in /proc/self/status (rustix
// `sched_getaffinity` needs its `thread` feature). Without the file, and
// off Linux, std's count.

// Go GODEBUG `containermaxprocs` (default 1 for a go.mod at `go 1.25` or
// later): 0 turns the cgroup limit off. The last valid setting wins.

/// Go internal/runtime/cgroup: the CPU limit of the cgroup that holds this
/// process.
#[cfg(target_os = "linux")]
mod cgroup {
    // Go: cgroup_linux.go OpenCPU and ReadCPULimit: quota / period of the
    // process's own CPU cgroup (not its parents). None when there is no
    // limit, or a file cannot be read or does not parse.

    // Go: cgroup.go:56 parseV1Number: "<value>\n".

    // Go: cgroup.go:72 parseV2Limit: "<quota> <period>\n". None for a quota
    // of "max" (no limit).

    // Go: cgroup.go:109 parseCPUCgroup: the CPU cgroup path in
    // /proc/self/cgroup and true for a v1 hierarchy. Lines are
    // `hierarchy-ID:controller-list:cgroup-path`. A v1 hierarchy with the
    // `cpu` controller wins over the v2 hierarchy (ID 0).
    fn parse_cpu_cgroup(text: &str) -> Option<(&str, bool)> {
        let mut v2 = None;
        for line in text.lines() {
            let (hierarchy, rest) = line.split_once(':')?;
            let (controllers, path) = rest.split_once(':')?;
            if !path.starts_with('/') {
                return None;
            }
            if hierarchy == "0" {
                v2 = Some(path);
            } else if controllers.split(',').any(|c| c == "cpu") {
                return Some((path, true));
            }
        }
        v2.map(|path| (path, false))
    }

    // Go: cgroup.go:232 parseCPUMount: the directory of `cgroup` under the
    // mount in /proc/self/mountinfo that holds it (`cgroup` with the `cpu`
    // option for v1, `cgroup2` for v2). Lines are `id parent major:minor
    // root mount-point options [optional...] - fstype source super-options`.

    // Go: cgroup.go:420 hasPathPrefix
    fn has_path_prefix(p: &str, prefix: &str) -> bool {
        prefix.len() == 1
            || p.starts_with(prefix)
                && (p.len() == prefix.len() || p.as_bytes()[prefix.len()] == b'/')
    }

    // Go: cgroup.go:452 unescapePath: Linux `show_path` writes `\`, space,
    // tab and newline as `\` and three octal digits.

    // Go: internal/runtime/cgroup/cgroup_test.go (cases of TestParseV1Number,
    // TestParseV2Limit, TestParseCPUCgroup and TestParseCPUMount).
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_cpu_limit_files() {
            assert_eq!(parse_v1_number("-1\n"), Some(-1));
            assert_eq!(parse_v1_number("50000\n"), Some(50000));
            assert_eq!(parse_v1_number("50000"), None);
            assert_eq!(parse_v2_limit("max 100000\n"), None);
            assert_eq!(parse_v2_limit("150000 100000\n"), Some(1.5));
            assert_eq!(parse_v2_limit("150000 100000"), None);
        }

        #[test]
        fn parse_cpu_cgroup_lines() {
            let v1 = "12:freezer:/\n3:cpu,cpuacct:/a/b\n0::/c\n";
            assert_eq!(parse_cpu_cgroup(v1), Some(("/a/b", true)));
            assert_eq!(
                parse_cpu_cgroup("1:memory:/m\n0::/c/d\n"),
                Some(("/c/d", false))
            );
            assert_eq!(parse_cpu_cgroup("1:memory:/m\n"), None);
            assert_eq!(parse_cpu_cgroup("0::c\n"), None);
        }

        #[test]
        fn parse_cpu_mount_lines() {
            let base = "22 1 8:1 / / rw,relatime - ext4 /dev/root rw\n\
                        21 22 0:20 / /sys rw,nosuid,nodev,noexec - sysfs sysfs rw\n";
            let v1 = "56 22 0:40 / /sys/fs/cgroup/cpu rw - cgroup cgroup rw,cpu,cpuacct\n\
                      59 22 0:43 / /sys/fs/cgroup/cpuset rw - cgroup cgroup rw,cpuset\n";
            let v2 = "25 21 0:22 / /sys/fs/cgroup rw,nosuid - cgroup2 cgroup2 rw\n";
            let mixed = format!("{base}{v2}{v1}");
            let cases: &[(&str, &str, &str, bool, Option<&str>)] = &[
                (
                    "v1",
                    &format!("{base}{v1}"),
                    "/",
                    true,
                    Some("/sys/fs/cgroup/cpu"),
                ),
                (
                    "v2",
                    &format!("{base}{v2}"),
                    "/",
                    false,
                    Some("/sys/fs/cgroup"),
                ),
                ("mixed", &mixed, "/", true, Some("/sys/fs/cgroup/cpu")),
                (
                    "mixed-choose-v2",
                    &mixed,
                    "/",
                    false,
                    Some("/sys/fs/cgroup"),
                ),
                (
                    "v2-escaped",
                    "25 21 0:22 / /sys/fs/cgroup/tab\\011tab rw - cgroup2 cgroup2 rw\n",
                    "/",
                    false,
                    Some("/sys/fs/cgroup/tab\ttab"),
                ),
                (
                    "non-root_mount",
                    "25 21 0:22 /sand /unrelated/cgroup1 rw - cgroup2 cgroup2 rw\n\
                     25 21 0:22 /sandbox/container/group /sys/fs/cgroup/mygroup rw - cgroup2 cgroup2 rw\n\
                     25 21 0:22 /sandbox /sys/fs/cgroup rw - cgroup2 cgroup2 rw\n\
                     25 21 0:22 / /ignored/second/match rw - cgroup2 cgroup2 rw\n",
                    "/sandbox/container",
                    false,
                    Some("/sys/fs/cgroup/container"),
                ),
                (
                    "v2-escaped-root",
                    "25 21 0:22 /tab\\011tab /sys/fs/cgroup rw - cgroup2 cgroup2 rw\n",
                    "/tab\ttab/container",
                    false,
                    Some("/sys/fs/cgroup/container"),
                ),
                (
                    "non-root_cgroup",
                    v2,
                    "/sandbox/container",
                    false,
                    Some("/sys/fs/cgroup/sandbox/container"),
                ),
                (
                    "out_of_namespace",
                    "1243 61 0:26 /../../.. /mnt rw shared:4 - cgroup2 cgroup2 rw\n\
                     29 22 0:26 /../../../.. /sys/fs/cgroup rw shared:4 - cgroup2 cgroup2 rw",
                    "/../../../../init.scope",
                    false,
                    Some("/sys/fs/cgroup/init.scope"),
                ),
                ("v1-not-mounted", v2, "/", true, None),
                (
                    "invalid-escape",
                    "25 21 0:22 /\\0 /x rw - cgroup2 cgroup2 rw\n",
                    "/",
                    false,
                    None,
                ),
            ];
            for &(name, text, cgroup, v1, want) in cases {
                assert_eq!(parse_cpu_mount(text, cgroup, v1).as_deref(), want, "{name}");
            }
        }
    }
}
