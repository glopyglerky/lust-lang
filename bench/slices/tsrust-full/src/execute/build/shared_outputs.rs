//! PORT: not in Go (determinism). Go builds the tasks of `tsc -b` on up to
//! `numRoutines` goroutines, and each task writes its outputs when its own
//! build ends. When two tasks that run at the same time write the same
//! file, or one writes a file that the other reads, the output depends on
//! which task ends first, so Go's output can change between runs. The port
//! makes the task programs one at a time on the orchestrator thread
//! (build_task.rs), so its task end order is not Go's. `build_all_tasks`
//! finishes the tasks in the order their checks end, except in a build
//! where `outputs_overlap` finds two tasks that can see each other's
//! writes. There every task finishes in build order, so the output is the
//! same in every run, and it is Go's output when Go's tasks end in build
//! order.
//!
//! For example, when two tasks A and B (in that order) write the same
//! `.d.ts`, and D references B while E references A: A ends first here, E
//! starts and parses A's file, and D gets that parse after B ends, because
//! the build host keeps the first parse of each `.d.ts` for the whole build
//! (Go `host.sourceFiles`). Go gives D A's file too when A ends first; when
//! B ends first, D and E both get B's file.

use crate::frontend::prelude::*;




/// The most links that `real_path` follows through names that do not
/// exist yet (Linux MAXSYMLINKS).
const MAX_LINKS: usize = 40;




/// The output area and the root file directories of one task, as path
/// keys (`PathKeys`). A directory key ends with '/', so a directory holds
/// each key that starts with its key. `beside_sources` is true when some
/// outputs of the task go next to its sources. `unknown` is true when a key
/// of the task cannot be trusted, so the area can hold any place.
struct Area {
    output_dirs: Vec<String>,
    build_info: Option<String>,
    root_dirs: Vec<String>,
    beside_sources: bool,
    unknown: bool,
}

impl Area {
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    OutputDir,
    OutputFile,
    RootDir,
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::vfs::osvfs_fs;

    fn area(output_dirs: &[&str], build_info: &str, root_dirs: &[&str]) -> Area {
        let keys = |keys: &[&str]| keys.iter().map(|key| key.to_string()).collect();
        Area {
            output_dirs: keys(output_dirs),
            build_info: (!build_info.is_empty()).then(|| build_info.to_string()),
            root_dirs: keys(root_dirs),
            beside_sources: false,
            unknown: false,
        }
    }

    /// The area of a task whose outputs go next to its sources in
    /// `root_dir`, with the build info `build_info`.
    fn beside(root_dir: &str, build_info: &str) -> Area {
        Area {
            beside_sources: true,
            ..area(&[root_dir], build_info, &[root_dir])
        }
    }

    /// The area (`Area::of`) of a solution config: no root files, project
    /// references and no outDir. It writes nothing.
    fn solution() -> Area {
        let config = new_parsed_command_line(
            Rc::new(CompilerOptions::default()),
            Vec::new(),
            Some(Vec::new()),
            ComparePathsOptions::default(),
        );
        let compare = ComparePathsOptions::default();
        Area::of(&config, &PathKeys::new(&osvfs_fs(), &compare))
    }

    #[test]
    fn separate_outputs_do_not_overlap() {
        // Packages that each write to their own dist (query-chain, wide).
        let areas = [
            area(
                &["/r/a/dist/", "/r/a/dist/"],
                "/r/a/dist/tsconfig.tsbuildinfo",
                &["/r/a/src/", "/r/a/"],
            ),
            area(
                &["/r/b/dist/"],
                "/r/b/dist/tsconfig.tsbuildinfo",
                &["/r/b/src/", "/r/b/"],
            ),
            area(&[], "/r/c/tsconfig.tsbuildinfo", &["/r/c/src/"]),
        ];
        assert!(!areas_overlap(&areas));
    }

    #[test]
    fn overlaps() {
        // A shared outDir, an outDir under another one, a build info in
        // another outDir, a root directory in another outDir, a shared
        // build info alone, two tasks whose outputs go next to their
        // sources in separate directories (they can import one file, as
        // the R149 reviewer's pX and pY import ../shared/u.ts), and an area
        // with a key that cannot be trusted.
        let pairs = [
            (
                area(&["/r/shared/"], "", &[]),
                area(&["/r/shared/"], "", &[]),
            ),
            (area(&["/r/dist/"], "", &[]), area(&["/r/dist/b/"], "", &[])),
            (
                area(&["/r/dist/"], "", &[]),
                area(&[], "/r/dist/b.tsbuildinfo", &[]),
            ),
            (
                area(&["/r/gen/"], "", &[]),
                area(&[], "", &["/r/gen/types/"]),
            ),
            (
                area(&[], "/r/x.tsbuildinfo", &[]),
                area(&[], "/r/x.tsbuildinfo", &[]),
            ),
            (
                beside("/r/pX/src/", "/r/x.tsbuildinfo"),
                beside("/r/pY/src/", "/r/y.tsbuildinfo"),
            ),
            (
                Area {
                    unknown: true,
                    ..area(&["/r/a/dist/"], "", &[])
                },
                area(&["/r/b/dist/"], "", &[]),
            ),
        ];
        for (a, b) in pairs {
            assert!(areas_overlap(&[a, b]));
        }
        // A directory whose name starts with another's name, an outDir
        // inside a root directory (the root files are only the ones there),
        // one task whose outputs go next to its sources beside one with an
        // outDir, and solutions (the R149 tscbfix1 skeptic's nested
        // solutions, and a solution beside a task whose outputs go next to
        // its sources).
        let pairs = [
            (
                area(&["/r/dist/"], "", &[]),
                area(&["/r/dist2/"], "/r/dist.tsbuildinfo", &[]),
            ),
            (area(&["/r/a/dist/"], "", &[]), area(&[], "", &["/r/a/"])),
            (
                beside("/r/pX/src/", "/r/x.tsbuildinfo"),
                area(&["/r/pY/dist/"], "/r/y.tsbuildinfo", &["/r/pY/src/"]),
            ),
            (solution(), solution()),
            (solution(), beside("/r/pX/src/", "/r/x.tsbuildinfo")),
        ];
        for (a, b) in pairs {
            assert!(!areas_overlap(&[a, b]));
        }
    }

    #[cfg(unix)]
    struct System {
        fs: Rc<dyn Fs>,
        current_directory: String,
    }

    #[cfg(unix)]
    impl ParseConfigHost for System {
        fn fs(&self) -> Rc<dyn Fs> {
            self.fs.clone()
        }
        fn get_current_directory(&self) -> String {
            self.current_directory.clone()
        }
    }

    /// (name, target) of each link that `overlap_on_disk` writes.
    #[cfg(unix)]
    const LINKS: [(&str, &str); 8] = [
        ("lnk", "."),
        ("lnkx", "pX"),
        ("ylink.tsbuildinfo", "shared.tsbuildinfo"),
        ("pY/dl", "../pX/dist"),
        ("pY/elsewhere", "../other/dist"),
        ("loop", "loop"),
        ("al", "d1/d2"),
        ("yb.tsbuildinfo", "al/../../shared.tsbuildinfo"),
    ];

    /// Writes the projects pX and pY (with `py_options`), the directory
    /// d1/d2, the file h1.tsbuildinfo with the hard link h2.tsbuildinfo, and
    /// the links in `LINKS` in a new temporary directory. Returns whether
    /// their outputs overlap and whether the build info prefetch leaves
    /// out pY's build info (`PathKeys::build_info_key`): it has the key of
    /// pX's build info, or a key that cannot be trusted. No output is
    /// written, so the links to outputs dangle.
    #[cfg(unix)]
    fn overlap_on_disk(label: &str, py_options: &str) -> (bool, bool) {
        let dir = std::env::temp_dir().join(format!(
            "ts_goport_shared_outputs_{label}_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        for (project, options) in [
            (
                "pX",
                r#""outDir": "dist", "tsBuildInfoFile": "../shared.tsbuildinfo""#,
            ),
            ("pY", py_options),
        ] {
            std::fs::create_dir_all(dir.join(project).join("src")).unwrap();
            std::fs::write(dir.join(project).join("src/m.ts"), "export const m = 1;\n").unwrap();
            std::fs::write(
                dir.join(project).join("tsconfig.json"),
                format!(r#"{{ "compilerOptions": {{ "composite": true, {options} }} }}"#),
            )
            .unwrap();
        }
        std::fs::create_dir_all(dir.join("d1/d2")).unwrap();
        std::fs::write(dir.join("h1.tsbuildinfo"), "{}").unwrap();
        std::fs::hard_link(dir.join("h1.tsbuildinfo"), dir.join("h2.tsbuildinfo")).unwrap();
        for (name, target) in LINKS {
            std::os::unix::fs::symlink(target, dir.join(name)).unwrap();
        }
        let cwd = dir.to_string_lossy().replace('\\', "/");
        let sys = System {
            fs: wrap_fs(osvfs_fs()),
            current_directory: cwd.clone(),
        };
        let configs: Vec<_> = ["pX", "pY"]
            .iter()
            .map(|project| {
                let (config, errors) = get_parsed_command_line_of_config_file(
                    &format!("{cwd}/{project}/tsconfig.json"),
                    None,
                    None,
                    &sys,
                    None,
                );
                assert!(errors.is_empty());
                Some(Rc::new(config.unwrap()))
            })
            .collect();
        let compare = ComparePathsOptions {
            current_directory: cwd,
            use_case_sensitive_file_names: true,
        };
        let overlap = outputs_overlap(&configs, &sys.fs, &compare);
        let keys = PathKeys::new(&sys.fs, &compare);
        let [x, y] = [0, 1]
            .map(|i| keys.build_info_key(&configs[i].as_ref().unwrap().get_build_info_file_name()));
        std::fs::remove_dir_all(&dir).unwrap();
        (overlap, x.is_none() || y.is_none() || x == y)
    }

    #[cfg(unix)]
    #[test]
    fn outputs_overlap_through_symbolic_links() {
        // The skeptic's bi2link: one build info, named through a link to
        // the root by pY. It does not exist yet. The R149 reviewer's case
        // for the build info prefetch: the two names have one key.
        assert_eq!(
            overlap_on_disk(
                "bi2link",
                r#""outDir": "dist", "tsBuildInfoFile": "../lnk/shared.tsbuildinfo""#,
            ),
            (true, true)
        );
        // The same through the link twice.
        assert_eq!(
            overlap_on_disk(
                "bi2link2",
                r#""outDir": "dist", "tsBuildInfoFile": "../lnk/lnk/shared.tsbuildinfo""#,
            ),
            (true, true)
        );
        // pY writes to pX's outDir through a link to pX.
        assert_eq!(
            overlap_on_disk(
                "outdir",
                r#""outDir": "../lnkx/dist", "tsBuildInfoFile": "../y.tsbuildinfo""#,
            ),
            (true, false)
        );
        // The int22 skeptic's dbi: pY's build info is a dangling link to
        // pX's build info.
        assert_eq!(
            overlap_on_disk(
                "dbi",
                r#""outDir": "dist", "tsBuildInfoFile": "../ylink.tsbuildinfo""#,
            ),
            (true, true)
        );
        // The int22 skeptic's dout: pY's outDir is a dangling link to pX's
        // outDir.
        assert_eq!(
            overlap_on_disk(
                "dout",
                r#""outDir": "dl", "tsBuildInfoFile": "../y.tsbuildinfo""#,
            ),
            (true, false)
        );
        // The tscbfix1 skeptic's skhard: pY's build info is a hard link to
        // another file. A write of that file changes it.
        assert_eq!(
            overlap_on_disk(
                "hard",
                r#""outDir": "dist", "tsBuildInfoFile": "../h2.tsbuildinfo""#,
            ),
            (true, true)
        );
        // The tscbfix1 skeptic's skdotdot: pY's build info is a dangling
        // link to al/../../shared.tsbuildinfo. As text that is outside the
        // root, but the OS goes from al (d1/d2) to d1 and then to the root,
        // so it is pX's build info. The key cannot be trusted.
        assert_eq!(
            overlap_on_disk(
                "dotdot",
                r#""outDir": "dist", "tsBuildInfoFile": "../yb.tsbuildinfo""#,
            ),
            (true, true)
        );
        // Separate outputs, each named through a link, do not overlap: a
        // link to the root, a dangling link to another place (its target
        // starts with `..`), and a link loop.
        assert_eq!(
            overlap_on_disk(
                "separate",
                r#""outDir": "../lnk/pY/dist", "tsBuildInfoFile": "../lnk/y.tsbuildinfo""#,
            ),
            (false, false)
        );
        assert_eq!(
            overlap_on_disk(
                "elsewhere",
                r#""outDir": "elsewhere", "tsBuildInfoFile": "../y.tsbuildinfo""#,
            ),
            (false, false)
        );
        assert_eq!(
            overlap_on_disk(
                "loop",
                r#""outDir": "../loop/dist", "tsBuildInfoFile": "../y.tsbuildinfo""#,
            ),
            (false, false)
        );
    }
}
