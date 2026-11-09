// macro: child_test, os_child_test
// source: crates/ts_goport/tests/go_baselines/project_lsp/mod.rs:25
// source: crates/ts_goport/tests/go_baselines/project_lsp/resolveahead_test.rs:31
// mode: test
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
pub mod support {
    pub mod child {
        pub fn run_test_in_child(name: &str, f: impl FnOnce()) {
            println!("child:{name}");
            f();
        }
        pub fn run_test_in_child_with_env(name: &str, env: &[(&str, &str)], f: impl FnOnce()) {
            println!("child-env:{name}:{} vars", env.len());
            f();
        }
    }
}
pub mod project_lsp {
    pub mod projecttestutil {
        pub fn install_fs_override() {}
    }
}
pub fn resolve_ahead_off() -> Option<String> { None }
pub fn with_a_worker<'a>(env: &[(&'a str, &'a str)]) -> Vec<(&'a str, &'a str)> {
    let mut v = env.to_vec();
    v.push(("WORKER", "1"));
    v
}

macro_rules! child_test {
    ($(#[$meta:meta])* fn $name:ident() $body:block) => {
        $(#[$meta])*
        #[test]
        fn $name() {
            let path = concat!(module_path!(), "::", stringify!($name));
            let test = path.split_once("::").map_or(path, |(_, rest)| rest);
            crate::support::child::run_test_in_child(test, || {
                crate::project_lsp::projecttestutil::install_fs_override();
                $body
            });
        }
    };
    // The child runs with the environment variables `$env` set.
    (env $env:expr; $(#[$meta:meta])* fn $name:ident() $body:block) => {
        $(#[$meta])*
        #[test]
        fn $name() {
            let path = concat!(module_path!(), "::", stringify!($name));
            let test = path.split_once("::").map_or(path, |(_, rest)| rest);
            crate::support::child::run_test_in_child_with_env(test, $env, || {
                crate::project_lsp::projecttestutil::install_fs_override();
                $body
            });
        }
    };
}

macro_rules! os_child_test {
    ($(#[$meta:meta])* fn $name:ident() $body:block) => {
        os_child_test!(env &[]; $(#[$meta])* fn $name() $body);
    };
    (env $env:expr; $(#[$meta:meta])* fn $name:ident() $body:block) => {
        $(#[$meta])*
        #[test]
        fn $name() {
            let path = concat!(module_path!(), "::", stringify!($name));
            let test = path.split_once("::").map_or(path, |(_, rest)| rest);
            if let Some(reason) = resolve_ahead_off() {
                eprintln!("{test}: skipped: {reason}");
                return;
            }
            crate::support::child::run_test_in_child_with_env(test, &with_a_worker($env), || $body);
        }
    };
}

child_test!(fn sample_child() { assert_eq!(1 + 1, 2); });
child_test!(env &[("K", "V")]; fn sample_child_env() { assert!(true); });
os_child_test!(fn sample_os_child() { assert_eq!(2 * 2, 4); });

fn main() {
    // The macros generate #[test] fns. In this bin build they are
    // stripped; the runner also builds this file with --test.
    println!("child-test fixture built");
}
