//! Reuse the upstream whole-request cache after a completed native contract.
//!
//! Admission uses the compiler's actual dependency records, never a source
//! classifier. Only successful root-only binaries enter this cache. A hit
//! regenerates Cargo dep-info from that proven root-only input set. Libraries,
//! imported artifacts, external files and observed environment variables still
//! compile normally. This does not retain a frontend across source edits.

use crate::driver::{Session, cache::{self, Cache}};
use crate::protocol::{Request, Response};
use std::path::PathBuf;

pub struct RequestCache {
    request: Request,
    marker: PathBuf,
}

const MAGIC: &[u8; 8] = b"LNREQ001";

fn fingerprint(bytes: &[u8]) -> [u8; 16] {
    let (lo, hi) = cache::hash128(&[bytes], 0);
    let mut out = [0; 16];
    out[..8].copy_from_slice(&lo.to_le_bytes());
    out[8..].copy_from_slice(&hi.to_le_bytes());
    out
}

fn response_fingerprint(response: &Response) -> [u8; 16] {
    let mut bytes = Vec::new();
    crate::protocol::write_response(&mut bytes, response).expect("writing a byte vector cannot fail");
    fingerprint(&bytes)
}

impl RequestCache {
    pub fn new(request: &Request, native_options: impl FnOnce() -> String) -> Option<Self> {
        if !cfg!(target_os = "linux") || request.session.krate.manifest_path.is_some()
            || std::env::var("LUST_NO_CACHE").is_ok_and(|v| !v.is_empty() && v != "0") {
            return None;
        }
        // Bind the complete native invocation, including the client's original
        // environment, shown path and output choices. Only its digest is saved;
        // the cache never persists a copy of the caller's environment.
        let encoded = crate::protocol::encode(request);
        // The upstream protocol identifies its sibling lust-core executable.
        // Cargo invokes this executable instead, so bind its actual filesystem
        // identity too. Replacing a native compiler at the same path must miss.
        use std::os::unix::fs::MetadataExt;
        let executable = std::env::current_exe().ok()?;
        // Linux's executing inode remains correct if an installer replaces the
        // path between current_exe and stat. This cache is Linux-qualified.
        let metadata = std::fs::metadata("/proc/self/exe").ok()?;
        if !metadata.is_file() { return None; }
        // TinyCC creates the output under the caller's umask. Preserve that
        // contract when an otherwise identical invocation tightens permissions.
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let mask = status.lines().find_map(|line| line.strip_prefix("Umask:"))?.trim();
        let mask = u32::from_str_radix(mask, 8).ok()?;
        let identity = format!("{executable:?}:{}:{}:{}:{}:{}:{}:{}", metadata.dev(), metadata.ino(), metadata.len(),
                               metadata.mtime(), metadata.mtime_nsec(), metadata.ctime(), metadata.ctime_nsec());
        let native_options = native_options();
        let (lo, hi) = cache::hash128(&[b"native-request-v1", env!("LUST_BUILD_HASH").as_bytes(), identity.as_bytes(), &mask.to_le_bytes(), native_options.as_bytes(), &encoded], 0);
        let tag = format!("{lo:016x}{hi:016x}");
        let mut request = request.clone();
        // Cargo supplied the effective arguments and environment, already
        // bound above. Upstream's file: namespace suppresses parent-manifest
        // discovery and build-script refresh inside its cache key. Lookup and
        // store must not independently run Cargo. Only this cache surrogate
        // changes; compilation keeps the original SessionKey and ABI.
        request.session.krate.package_id = format!("file:native-request-v1:{tag}");
        let marker = cache::cache_dir(&request.file).join(format!("req-native-{tag}"));
        Some(Self { request, marker })
    }

    pub fn lookup(&self) -> Option<Response> {
        // A first cold compile does not open/hash the TinyCC cache a second
        // time. Admission records are written only after a complete success.
        let marker = std::fs::read(&self.marker).ok()?;
        if marker.len() != 60 || marker.get(..8) != Some(MAGIC) {
            return None;
        }
        let source = std::fs::read(&self.request.file).ok()?;
        if marker[8..24] != fingerprint(&source) {
            return None;
        }
        let cache = Cache::open(&self.request.file)?;
        // Upstream lookup copies the executable. Stage that copy so corrupt
        // cache data cannot replace the user's previous working executable.
        let stage = crate::cargo_cli::temporary_directory().ok()?;
        let mut request = self.request.clone();
        request.out = stage.join("cached-program");
        let answer = (|| {
            let response = cache.lookup(&request, &source, std::time::Instant::now())?;
            if response.code != 0 || response.ran || marker[40..56] != response_fingerprint(&response) {
                return None;
            }
            let artifact = std::fs::read(&request.out).ok()?;
            if marker[24..40] != fingerprint(&artifact) {
                return None;
            }
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::metadata(&request.out).ok()?.permissions();
            let expected_mode = u32::from_le_bytes(marker[56..60].try_into().ok()?);
            if expected_mode & 0o111 == 0 || permissions.mode() & 0o7777 != expected_mode {
                return None;
            }
            // The cache directory can be on another filesystem. Copy into a
            // private file beside the destination, then publish atomically.
            let parent = self.request.out.parent()?;
            let name = format!(".lustr-cache-{}", stage.file_name()?.to_str()?);
            let publish = parent.join(name);
            let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&publish).ok()?;
            use std::io::Write;
            let result = file.write_all(&artifact).and_then(|_| {
                file.set_permissions(permissions)?;
                std::fs::rename(&publish, &self.request.out)
            });
            let _ = std::fs::remove_file(&publish);
            result.ok()?;
            Some(response)
        })();
        let _ = std::fs::remove_dir_all(stage);
        answer
    }

    pub fn store(&self, session: &Session, additional: &[crate::expand::Dep], response: &Response) {
        if response.code != 0 || response.ran || !response.stderr.is_empty() {
            return;
        }
        let Some(cache) = &session.disk_cache else { return };
        // A filename scanner is insufficient: macros can spell outside reads
        // with whitespace, comments or expansion. Inspect their actual records.
        if session.front.chunks.iter().flat_map(|chunk| &chunk.expand_deps).chain(additional)
            .any(|dep| matches!(dep, crate::expand::Dep::File { .. } | crate::expand::Dep::Env { .. }))
        {
            return;
        }
        if session.front.ast_gen != 0 || session.front.map.file_count() != 1
            || session.front.map.path(crate::diag::FileId(0)) != self.request.file.to_string_lossy()
            || session.front.resolve_cache.source_paths().any(|path| path != self.request.file)
        {
            return;
        }
        let Ok(source) = std::fs::read(&self.request.file) else { return };
        if session.front.src.as_bytes() != source {
            return; // Includes a source edited while this compile ran.
        }
        let Ok(artifact) = std::fs::read(&self.request.out) else { return };
        use std::os::unix::fs::PermissionsExt;
        let Ok(metadata) = std::fs::metadata(&self.request.out) else { return };
        let mode = metadata.permissions().mode() & 0o7777;
        if !metadata.is_file() || mode & 0o111 == 0 { return; }
        let mut marker = Vec::with_capacity(60);
        marker.extend_from_slice(MAGIC);
        marker.extend_from_slice(&fingerprint(&source));
        marker.extend_from_slice(&fingerprint(&artifact));
        marker.extend_from_slice(&response_fingerprint(response));
        marker.extend_from_slice(&mode.to_le_bytes());
        // Reuse the identity the actual compile opened. All native artifact and
        // dep-info checks have already succeeded before this method is called.
        cache.store(&self.request, &source, response);
        let temporary = self.marker.with_extension(format!("tmp{}", std::process::id()));
        if std::fs::write(&temporary, marker).is_ok() {
            let _ = std::fs::rename(&temporary, &self.marker);
        }
        let _ = std::fs::remove_file(temporary);
    }
}
