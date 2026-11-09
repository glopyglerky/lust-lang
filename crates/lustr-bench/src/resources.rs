//! Fresh-process Linux RUSAGE_CHILDREN protocol. No unsafe code in owned logic.
use super::*;
use nix::sys::{
    resource::{UsageWho, getrusage},
    signal::{Signal, killpg},
    time::TimeValLike,
};
use nix::unistd::{getpgrp, getpid};
use std::process::ExitStatus;

pub const SCOPE: &str = "linux-rusage-children-v1: CPU sums waited children; descendants only through fully waited ancestry; peak RSS is largest waited process in KiB, not aggregate tree memory";
#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    #[default]
    Disabled,
    LinuxWaited,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub argv: Vec<Arg>,
    pub env: Vec<(Arg, Arg)>,
    pub cwd: PathBuf,
    pub collector_sha256: String,
    pub report_path: PathBuf,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    pub schema: u32,
    pub scope: String,
    pub request_sha256: String,
    pub collector_sha256: String,
    pub collector_pid: u32,
    pub raw_status: i32,
    pub user_microseconds: i64,
    pub system_microseconds: i64,
    pub peak_rss_kib: i64,
    pub collector_user_microseconds: i64,
    pub collector_system_microseconds: i64,
    pub payload_wall_seconds: f64,
}
pub fn metadata(mode: Mode) -> Value {
    if mode == Mode::Disabled {
        return json!({"mode":"disabled","availability":"not measured"});
    }
    json!({"mode":"linux-waited","scope":SCOPE,"completion_protocol":"v2: collector exits zero after publishing usage; payload exit/signal comes from the validated raw wait status","energy":"not measured","aggregate_rss":"not measured","resource_thresholds":"none; cannot satisfy aggregate memory gates","wall_protocol":"outer wall includes collector startup, payload and report; collector CPU reported separately; identical protocol required for all lanes","collector":"reexec runner; identity bound to runner_sha256 and each request"})
}
pub fn collect(request: &Path, report: &Path) -> R<i32> {
    let group = getpgrp();
    if group != getpid() {
        return fail("collector must lead its process group");
    }
    if report.try_exists()? {
        return fail("resource report already exists");
    }
    // FD 0 is a lease, never the payload stdin. FD 1/2 stay inherited byte streams.
    // Closing the parent's pipe kills this entire owned process group, including us.
    std::thread::spawn(move || {
        let mut byte = [0];
        let _ = std::io::stdin().read(&mut byte);
        let _ = killpg(group, Signal::SIGKILL);
    });
    let bytes = fs::read(request)?;
    let req: Request = serde_json::from_slice(&bytes)?;
    if req.report_path != report {
        return fail("resource report path mismatch");
    }
    if hash(&std::env::current_exe()?)? != req.collector_sha256 {
        return fail("collector identity changed");
    }
    let args = req.argv.iter().map(Arg::os).collect::<R<Vec<_>>>()?;
    let vars = req.env.iter().map(|(k, v)| Ok((k.os()?, v.os()?))).collect::<R<Vec<_>>>()?;
    let start = Instant::now();
    let status = Command::new(args.first().ok_or("empty payload")?)
        .args(&args[1..])
        .current_dir(req.cwd)
        .env_clear()
        .envs(vars)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    let wall = start.elapsed().as_secs_f64();
    let children = getrusage(UsageWho::RUSAGE_CHILDREN)?;
    let own = getrusage(UsageWho::RUSAGE_SELF)?;
    let usage = Usage {
        schema: 2,
        scope: SCOPE.into(),
        request_sha256: hex(&Sha256::digest(&bytes)),
        collector_sha256: req.collector_sha256,
        collector_pid: std::process::id(),
        raw_status: status.into_raw(),
        user_microseconds: children.user_time().num_microseconds(),
        system_microseconds: children.system_time().num_microseconds(),
        peak_rss_kib: children.max_rss(),
        collector_user_microseconds: own.user_time().num_microseconds(),
        collector_system_microseconds: own.system_time().num_microseconds(),
        payload_wall_seconds: wall,
    };
    publish(report, &serde_json::to_vec(&usage)?)?;
    // Collector completion and payload success are different protocol fields.
    // Reporting the wait status avoids changing inherited signal dispositions.
    Ok(0)
}
pub fn read(report: &Path, request_hash: &str, collector_hash: &str, pid: u32, wrapper: ExitStatus) -> R<Usage> {
    let u: Usage = serde_json::from_slice(&fs::read(report)?)?;
    let status = ExitStatus::from_raw(u.raw_status);
    if u.schema != 2
        || u.scope != SCOPE
        || u.request_sha256 != request_hash
        || u.collector_sha256 != collector_hash
        || u.collector_pid != pid
        || u.user_microseconds < 0
        || u.system_microseconds < 0
        || u.peak_rss_kib < 0
        || u.collector_user_microseconds < 0
        || u.collector_system_microseconds < 0
        || !u.payload_wall_seconds.is_finite()
        || u.payload_wall_seconds < 0.0
        || !(status.code().is_some() || status.signal().is_some())
        || (status.code().is_some() && u.raw_status & 255 != 0)
        || u.raw_status < 0
        || u.raw_status > 65535
        || status.signal().is_some_and(|signal| !(1..=64).contains(&signal) || u.raw_status & !255 != 0)
        || !wrapper.success()
    {
        return fail("invalid or mismatched resource report");
    }
    Ok(u)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_malformed_mismatched_and_status_inconsistent_reports_fail_closed() {
        let Some(root) = std::env::var_os("LUSTR_BENCH_RESOURCE_UNIT_OUTPUT") else {
            return;
        };
        let root = PathBuf::from(root);
        fs::create_dir(&root).unwrap();
        let path = root.join("usage.json");
        let ok = ExitStatus::from_raw(0);
        assert!(read(&path, "req", "collector", 42, ok).is_err());
        fs::write(&path, b"{").unwrap();
        assert!(read(&path, "req", "collector", 42, ok).is_err());
        let valid = json!({"schema":2,"scope":SCOPE,"request_sha256":"req","collector_sha256":"collector","collector_pid":42,"raw_status":0,"user_microseconds":1,"system_microseconds":2,"peak_rss_kib":3,"collector_user_microseconds":4,"collector_system_microseconds":5,"payload_wall_seconds":0.01});
        fs::write(&path, serde_json::to_vec(&valid).unwrap()).unwrap();
        assert!(read(&path, "req", "collector", 42, ok).is_ok());
        for (key, value) in [
            ("schema", json!(1)),
            ("scope", json!("aggregate")),
            ("request_sha256", json!("old")),
            ("collector_sha256", json!("old")),
            ("collector_pid", json!(41)),
            ("raw_status", json!(65535)),
            ("raw_status", json!(65536)),
            ("peak_rss_kib", json!(-1)),
            ("user_microseconds", json!(-1)),
        ] {
            let mut bad = valid.clone();
            bad[key] = value;
            fs::write(&path, serde_json::to_vec(&bad).unwrap()).unwrap();
            assert!(read(&path, "req", "collector", 42, ok).is_err(), "{key}");
        }
        for raw in [13, 15, 7 << 8] {
            let mut payload = valid.clone();
            payload["raw_status"] = json!(raw);
            fs::write(&path, serde_json::to_vec(&payload).unwrap()).unwrap();
            assert_eq!(read(&path, "req", "collector", 42, ok).unwrap().raw_status, raw);
            assert!(read(&path, "req", "collector", 42, ExitStatus::from_raw(raw)).is_err());
            assert!(read(&path, "req", "collector", 42, ExitStatus::from_raw(1 << 8)).is_err());
        }
    }
}
