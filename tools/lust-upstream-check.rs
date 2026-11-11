use std::path::{Component, Path};
use std::process::Command;

struct Pin {
    revision: String,
    upstream_tree: String,
    tree: String,
    path: String,
}

fn pin() -> Result<Pin, String> {
    let mut values = std::collections::BTreeMap::new();
    for line in include_str!("../upstream/lust.pin").lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or("invalid upstream pin")?;
        let value = value.trim().strip_prefix('"').and_then(|value| value.strip_suffix('"')).ok_or("invalid upstream pin value")?;
        if values.insert(key.trim(), value).is_some() {
            return Err("duplicate upstream pin field".into());
        }
    }
    if values.len() != 5 || values.get("repository") != Some(&"https://github.com/t3dotgg/lust-lang") {
        return Err("unexpected upstream pin fields or repository".into());
    }
    let revision = values.get("revision").ok_or("missing upstream revision")?.to_string();
    let upstream_tree = values.get("upstream_tree").ok_or("missing upstream tree")?.to_string();
    let tree = values.get("tree").ok_or("missing accepted fork tree")?.to_string();
    let path = values.get("path").ok_or("missing upstream path")?.to_string();
    if [&revision, &upstream_tree, &tree].iter().any(|value| value.len() != 40 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        || path.is_empty()
        || !Path::new(&path).components().all(|component| matches!(component, Component::Normal(_)))
    {
        return Err("invalid upstream object identity or path".into());
    }
    Ok(Pin { revision, upstream_tree, tree, path })
}

fn git(repo: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git").arg("-C").arg(repo).args(args).output().map_err(|error| format!("cannot execute Git: {error}"))?;
    if !output.status.success() {
        return Err("native source check failed; inspect the scoped Git diff and preserve both upstream source and fork changes".into());
    }
    Ok(output.stdout)
}

fn check(repo: &Path, pin: &Pin, index: bool) -> Result<(), String> {
    let tree = if index {
        git(repo, &["write-tree", &format!("--prefix={}/", pin.path)])?
    } else {
        git(repo, &["rev-parse", "--verify", &format!("HEAD:{}", pin.path)])?
    };
    if tree.strip_suffix(b"\n").unwrap_or(&tree) != pin.tree.as_bytes() {
        return Err(
            "native source differs from the accepted fork tree; preserve working fixes and record reviewed changes with a pin update".into()
        );
    }
    git(
        repo,
        &["-c", "core.filemode=true", "diff", "--no-ext-diff", "--ignore-submodules=none", "--quiet", "--exit-code", "--", &pin.path],
    )?;
    if !index {
        git(repo, &["diff", "--cached", "--no-ext-diff", "--ignore-submodules=none", "--quiet", "--exit-code", "HEAD", "--", &pin.path])?;
    }
    if !git(repo, &["ls-files", "--others", "--exclude-standard", "-z", "--", &pin.path])?.is_empty() {
        return Err("untracked files in the native source tree; review and record intentional additions before updating the fork pin".into());
    }
    Ok(())
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let index = match args.as_slice() {
        [] => false,
        [arg] if arg == "--index" => true,
        _ => {
            eprintln!("usage: lust-upstream-check [--index]");
            std::process::exit(2);
        }
    };
    let result = pin().and_then(|pin| {
        check(Path::new("."), &pin, index)?;
        println!(
            "Native source matches accepted fork tree {}; upstream {} has tree {}",
            pin.tree, pin.revision, pin.upstream_tree
        );
        Ok(())
    });
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SERIAL: AtomicU64 = AtomicU64::new(0);

    struct Fixture(std::path::PathBuf);

    impl Fixture {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("lust-upstream-check-{}-{}", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)));
            fs::create_dir(&path).unwrap();
            git(&path, &["init", "-q"]).unwrap();
            fs::create_dir(path.join("vendor")).unwrap();
            fs::write(path.join("vendor/source.rs"), "fn upstream() {}\n").unwrap();
            fs::write(path.join("owned.rs"), "fn owned() {}\n").unwrap();
            git(&path, &["add", "--", "vendor", "owned.rs"]).unwrap();
            let fixture = Self(path);
            fixture.commit();
            fixture
        }

        fn commit(&self) {
            let output = Command::new("git")
                .arg("-C")
                .arg(&self.0)
                .args(["-c", "commit.gpgsign=false", "commit", "-q", "-m", "source fixture"])
                .env("GIT_AUTHOR_NAME", "glopyglerky")
                .env("GIT_COMMITTER_NAME", "glopyglerky")
                .env("GIT_AUTHOR_EMAIL", "189872235+glopyglerky@users.noreply.github.com")
                .env("GIT_COMMITTER_EMAIL", "189872235+glopyglerky@users.noreply.github.com")
                .output()
                .unwrap();
            assert!(output.status.success());
        }

        fn pin(&self) -> Pin {
            Pin {
                revision: "0".repeat(40),
                upstream_tree: String::from_utf8(git(&self.0, &["rev-parse", "HEAD:vendor"]).unwrap()).unwrap().trim().into(),
                tree: String::from_utf8(git(&self.0, &["rev-parse", "HEAD:vendor"]).unwrap()).unwrap().trim().into(),
                path: "vendor".into(),
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn detects_source_edits_and_additions_while_preserving_owned_work() {
        let fixture = Fixture::new();
        let pin = fixture.pin();
        check(&fixture.0, &pin, false).unwrap();
        fs::write(fixture.0.join("owned.rs"), "fn independently_owned() {}\n").unwrap();
        check(&fixture.0, &pin, false).unwrap();
        fs::write(fixture.0.join("vendor/source.rs"), "fn changed() {}\n").unwrap();
        assert!(check(&fixture.0, &pin, false).is_err());
        git(&fixture.0, &["add", "--", "vendor"]).unwrap();
        assert!(check(&fixture.0, &pin, false).is_err());
        assert!(check(&fixture.0, &pin, true).is_err());
        fs::write(fixture.0.join("vendor/source.rs"), "fn upstream() {}\n").unwrap();
        git(&fixture.0, &["add", "--", "vendor"]).unwrap();
        fs::write(fixture.0.join("vendor/extra.rs"), "fn extra() {}\n").unwrap();
        assert!(check(&fixture.0, &pin, false).is_err());
        fs::remove_file(fixture.0.join("vendor/extra.rs")).unwrap();
        check(&fixture.0, &pin, false).unwrap();
        assert_eq!(fs::read_to_string(fixture.0.join("owned.rs")).unwrap(), "fn independently_owned() {}\n");
    }

    #[test]
    fn checks_a_staged_restoration_before_commit() {
        let fixture = Fixture::new();
        let pin = fixture.pin();
        fs::write(fixture.0.join("vendor/source.rs"), "fn changed() {}\n").unwrap();
        git(&fixture.0, &["add", "--", "vendor"]).unwrap();
        fixture.commit();
        assert!(check(&fixture.0, &pin, false).is_err());
        fs::write(fixture.0.join("vendor/source.rs"), "fn upstream() {}\n").unwrap();
        git(&fixture.0, &["add", "--", "vendor"]).unwrap();
        assert!(check(&fixture.0, &pin, false).is_err());
        check(&fixture.0, &pin, true).unwrap();
    }

    #[test]
    fn accepts_recorded_fork_fixes_and_rejects_reset_to_pristine_upstream() {
        let fixture = Fixture::new();
        let upstream = fixture.pin();
        fs::write(fixture.0.join("vendor/source.rs"), "fn upstream_with_required_fix() {}\n").unwrap();
        git(&fixture.0, &["add", "--", "vendor"]).unwrap();
        fixture.commit();
        let mut fork = fixture.pin();
        fork.upstream_tree = upstream.tree.clone();
        check(&fixture.0, &fork, false).unwrap();

        fs::write(fixture.0.join("vendor/source.rs"), "fn upstream() {}\n").unwrap();
        git(&fixture.0, &["add", "--", "vendor"]).unwrap();
        assert!(check(&fixture.0, &fork, true).is_err());
        fixture.commit();
        assert!(check(&fixture.0, &fork, false).is_err());

        fs::write(fixture.0.join("vendor/source.rs"), "fn upstream_with_required_fix_and_extension() {}\n").unwrap();
        git(&fixture.0, &["add", "--", "vendor"]).unwrap();
        fixture.commit();
        let mut reviewed = fixture.pin();
        reviewed.upstream_tree = upstream.tree;
        check(&fixture.0, &reviewed, false).unwrap();
    }

}
