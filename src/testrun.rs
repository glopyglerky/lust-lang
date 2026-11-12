// `lust test <file.rs> [filter]`: finds `#[test]` functions, builds one test binary, and runs
// each test in its own child process with a timeout. A panicking test ends its child only (it
// unwinds, as under `rustc --test`, and exits 101), never the run.
//
// Tests are discovered from the native resolver's AST after checking the original crate
// with cfg(test). The backend adds a hosted dispatcher that calls those function slots.
// Source files, module paths, macro locations and the crate's no_std prelude stay unchanged.
// A user main remains an ordinary function, as with rustc --test. Inline and file modules,
// including private modules and macro-generated tests, follow ordinary crate resolution.
//
// Exit status follows cargo test: 0 when every selected test passes, 101 when the build fails or
// any test fails or times out, 2 for bad arguments or an unreadable file. A filtered test is
// never reported as a pass; it is counted in `filtered_out`. A `#[ignore]` test is reported as
// ignored and never runs. A `#[should_panic]` test passes only when it panics with the
// expected text, when one is written.
use crate::ast::{Ast, Attr, AttrInput, Expr, ItemKind, TyExpr};
use crate::diag::{self, Diag};
use crate::driver::{self, Cmd};
use crate::intern::{Interner, Sym};
use crate::lexer;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const USAGE: &str = "usage: lust test <file.rs> [filter] [--json] [--list] [--timeout SECONDS] [--max-errors N] [crate selection flags]";

struct Opts {
    file: PathBuf,
    filter: Option<String>,
    json: bool,
    list: bool,
    timeout: Duration,
    max_errors: usize,
    crate_args: Vec<String>,
}

/// Parses `lust test` arguments (without the word `test`).
fn parse_opts(args: &[String]) -> Result<Opts, String> {
    let (mut file, mut filter, mut json, mut list) = (None, None, false, false);
    let (mut timeout, mut max_errors) = (Duration::from_secs(10), diag::DEFAULT_MAX_ERRORS);
    let mut crate_args = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => json = true,
            "--list" => list = true,
            "--timeout" => timeout = parse_secs(it.next().ok_or(USAGE)?)?,
            "--max-errors" => max_errors = it.next().and_then(|v| v.parse().ok()).ok_or(USAGE)?,
            _ if a.starts_with("--timeout=") => timeout = parse_secs(&a["--timeout=".len()..].to_string())?,
            _ if a.starts_with("--max-errors=") => max_errors = a["--max-errors=".len()..].parse().map_err(|_| USAGE.to_string())?,
            "--manifest-path" | "--package" | "-p" | "--bin" | "--features" | "--target" | "-C" => {
                crate_args.push(a.clone());
                crate_args.push(it.next().ok_or(USAGE)?.clone());
            }
            "--lib" | "--no-default-features" | "--all-features" | "--offline" | "--locked" | "--frozen" => crate_args.push(a.clone()),
            _ if ["--manifest-path=", "--package=", "--bin=", "--features=", "--target="].iter().any(|prefix| a.starts_with(prefix)) => {
                crate_args.push(a.clone());
            }
            _ if a.starts_with('-') => return Err(USAGE.into()),
            _ if file.is_none() => file = Some(PathBuf::from(a)),
            _ if filter.is_none() => filter = Some(a.clone()),
            _ => return Err(USAGE.into()),
        }
    }
    Ok(Opts { file: file.ok_or(USAGE)?, filter, json, list, timeout, max_errors, crate_args })
}

fn parse_secs(s: &String) -> Result<Duration, String> {
    let v: f64 = s.parse().map_err(|_| USAGE.to_string())?;
    if v <= 0.0 {
        return Err(USAGE.into());
    }
    Ok(Duration::from_secs_f64(v))
}

/// One test case result.
struct Outcome {
    name: String,
    status: &'static str, // "pass", "fail", "timeout", or "ignored"
    duration_ms: f64,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Runs the binary once for test `index`, capturing output and enforcing the timeout.
fn run_one(bin: &PathBuf, index: usize, name: &str, timeout: Duration) -> Outcome {
    let start = Instant::now();
    let mut cmd = Command::new(bin);
    cmd.args(std::iter::repeat_n("x", index + 1)).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return Outcome {
                name: name.into(),
                status: "fail",
                duration_ms: millis(start),
                exit_code: None,
                stdout: String::new(),
                stderr: format!("cannot run the test binary: {e}"),
            };
        }
    };
    let mut out_pipe = child.stdout.take().expect("piped");
    let mut err_pipe = child.stderr.take().expect("piped");
    let out_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = out_pipe.read_to_end(&mut buf);
        buf
    });
    let err_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = err_pipe.read_to_end(&mut buf);
        buf
    });
    let (status, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(s)) => break (Some(s), false),
            Ok(None) if start.elapsed() < timeout => std::thread::sleep(Duration::from_millis(2)),
            Ok(None) => {
                let _ = child.kill();
                break (child.wait().ok(), true);
            }
            Err(_) => break (child.wait().ok(), false),
        }
    };
    let stdout = String::from_utf8_lossy(&out_thread.join().unwrap_or_default()).into_owned();
    let stderr = String::from_utf8_lossy(&err_thread.join().unwrap_or_default()).into_owned();
    let exit_code = status.and_then(|s| s.code());
    let status = if timed_out {
        "timeout"
    } else if exit_code == Some(0) {
        "pass"
    } else {
        "fail"
    };
    Outcome { name: name.into(), status, duration_ms: millis(start), exit_code, stdout, stderr }
}

fn millis(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

/// One `#[test]` fn and its harness attributes. `should_panic` is
/// `Some(expected)` for `#[should_panic]`, where `expected` is the
/// panic text it must contain, or `None` inside for the bare form.
#[derive(Clone)]
struct TestCase {
    /// The path from the crate root (`tests::adds`), as rustc --test names it.
    name: String,
    /// The function's native resolved name, independent of its visibility.
    symbol: Sym,
    ignored: bool,
    should_panic: Option<Option<String>>,
}

/// Finds tests in the already resolved native AST. The resolver has applied cfg,
/// expanded macros, loaded module files and retained each definition's source span.
fn find_tests(
    ast: &Ast,
    interner: &Interner,
    display_name: impl Fn(Sym) -> Sym,
    tests: &mut Vec<TestCase>,
    bad: &mut Vec<Diag>,
) {
    for item in &ast.items {
        match &item.kind {
            ItemKind::Fn(def) if def.is_test => {
                let name = interner.get(display_name(def.name)).to_string();
                let ret_ok = def.ret.is_none_or(|t| matches!(ast.ty(t), TyExpr::Unit));
                if def.self_param.is_some() || !def.params.is_empty() || !def.generics.params.is_empty() || !ret_ok {
                    bad.push(Diag::new(
                        "L0110",
                        def.name_span,
                        format!("test function `{name}` must take no arguments, have no generic parameters and return `()`"),
                        "Write a non-generic test function with no arguments that returns `()`.",
                    ));
                } else {
                    let (ignored, should_panic) = harness_attrs(&item.attrs, ast, interner);
                    tests.push(TestCase { name, symbol: def.name, ignored, should_panic });
                }
            }
            ItemKind::Impl(imp) => {
                for def in &imp.fns {
                    if def.is_test {
                        bad.push(diag::unsupported(def.name_span, "#[test] functions inside impl blocks"));
                    }
                }
            }
            _ => {}
        }
    }
}

/// The string literal in an attribute input, whichever of the three
/// stored shapes holds it (`= "x"` trees, a parenthesized list, or an
/// `Eq` expression from a cfg_attr expansion).
fn attr_str(input: &AttrInput, ast: &Ast) -> Option<String> {
    fn in_trees(trees: &[lexer::TokenTree], ast: &Ast) -> Option<String> {
        trees.iter().find_map(|t| match t {
            lexer::TokenTree::Token(tok) => match tok.tok {
                lexer::Tok::Str(i, _) => ast.strings.get(i as usize).cloned(),
                _ => None,
            },
            lexer::TokenTree::Delim(_, inner, _) => in_trees(inner, ast),
        })
    }
    match input {
        AttrInput::None => None,
        AttrInput::Eq(e) => match ast.expr(*e) {
            Expr::Str(i) => ast.strings.get(*i as usize).cloned(),
            _ => None,
        },
        AttrInput::List(_, trees) => in_trees(trees, ast),
    }
}

/// Reads `#[ignore]` and the first `#[should_panic]` from one item.
/// A second `should_panic` does not change the result, as in rustc
/// (probed: the first expected text wins). The parser already judged
/// each written form, so a bare `should_panic` and one whose string
/// this reader cannot find both count as no expected text.
fn harness_attrs(attrs: &[Attr], ast: &Ast, interner: &Interner) -> (bool, Option<Option<String>>) {
    let mut ignored = false;
    let mut should_panic = None;
    for a in attrs {
        if a.path.len() != 1 {
            continue;
        }
        match interner.get(a.path[0].name) {
            "ignore" => ignored = true,
            "should_panic" if should_panic.is_none() => should_panic = Some(attr_str(&a.input, ast)),
            _ => {}
        }
    }
    (ignored, should_panic)
}

/// The JSON summary record printed last in `--json` mode.
fn summary_json(file: &str, hash: &str, build: &str, outcomes: &[Outcome], filtered_out: usize, start: Instant) -> String {
    let count = |s: &str| outcomes.iter().filter(|o| o.status == s).count();
    format!(
        "{{\"schema\":{},\"kind\":\"summary\",\"file\":{},\"source_hash\":{},\"build\":\"{}\",\"total\":{},\"passed\":{},\"failed\":{},\"timed_out\":{},\"ignored\":{},\"filtered_out\":{},\"duration_ms\":{:.3}}}",
        diag::SCHEMA,
        diag::json_str(file),
        diag::json_str(hash),
        build,
        outcomes.len(),
        count("pass"),
        count("fail"),
        count("timeout"),
        count("ignored"),
        filtered_out,
        millis(start)
    )
}

/// Entry point for `lust test`. Returns the process exit code.
pub fn run(args: &[String]) -> i32 {
    let opts = match parse_opts(args) {
        Ok(o) => o,
        Err(u) => {
            eprintln!("{u}");
            return 2;
        }
    };
    let start = Instant::now();
    let abs = |p: &PathBuf| std::path::absolute(p).unwrap_or_else(|_| p.clone());
    let file = abs(&opts.file);
    let path = file.to_string_lossy().into_owned();
    let src = match std::fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {path}: {e}");
            return 2;
        }
    };
    // Select and check the real crate through the normal compiler pipeline. Test
    // mode preserves its Cargo features, declared cfg names and dependency graph.
    let mut crate_args = vec!["check".to_string(), opts.file.to_string_lossy().into_owned()];
    crate_args.extend(opts.crate_args.iter().cloned());
    let Some(crate::protocol::Invocation::Compile { mut req, .. }) = crate::protocol::parse_args(&crate_args) else {
        eprintln!("{USAGE}");
        return 2;
    };
    let dir = std::env::temp_dir().join(format!("lust-test-{}", std::process::id()));
    let bin = dir.join("testbin");
    req.session.krate.test = true;
    req.harness = true;
    req.json = opts.json;
    req.max_errors = opts.max_errors;
    req.out = bin.clone();
    let mut session = driver::Session::new(dir.join("obj"), false, req.session.clone());
    let check = driver::compile(&req, &mut session);
    // Cargo target selection happens inside the native resolver, after argument
    // parsing. Metadata must identify those checked bytes, even when --lib or
    // --bin selected a different root than the explicit positional file.
    let (path, src) = session.harness_root.take().map_or((path, src), |(path, source)| (path.to_string_lossy().into_owned(), source));
    let hash = diag::source_hash(&src);
    let fail_build = |text: String, filtered_out: usize| -> i32 {
        if opts.json {
            print!("{text}");
            println!("{}", summary_json(&path, &hash, "failed", &[], filtered_out, start));
        } else {
            eprint!("{text}");
        }
        101
    };
    if check.code != 0 {
        let _ = std::fs::remove_dir_all(&dir);
        print!("{}", check.stdout);
        eprint!("{}", check.stderr);
        if opts.json {
            println!("{}", summary_json(&path, &hash, "failed", &[], 0, start));
        }
        return 101;
    }
    let mut tests = Vec::new();
    let mut bad = Vec::new();
    find_tests(
        &session.front.ast,
        &session.front.interner,
        |name| session.harness_names.get(&name).copied().unwrap_or(name),
        &mut tests,
        &mut bad,
    );
    if !bad.is_empty() {
        let text = diag::render(&mut bad, &session.front.map, &src, opts.json, opts.max_errors);
        let _ = std::fs::remove_dir_all(&dir);
        return fail_build(text, 0);
    }
    let selected: Vec<TestCase> = match &opts.filter {
        Some(f) => tests.iter().filter(|t| t.name.contains(f.as_str())).cloned().collect(),
        None => tests.clone(),
    };
    let filtered_out = tests.len() - selected.len();
    if opts.list {
        if opts.json {
            let names: Vec<String> = selected.iter().map(|t| diag::json_str(&t.name)).collect();
            println!(
                "{{\"schema\":{},\"kind\":\"list\",\"file\":{},\"source_hash\":{},\"tests\":[{}],\"filtered_out\":{}}}",
                diag::SCHEMA,
                diag::json_str(&path),
                diag::json_str(&hash),
                names.join(","),
                filtered_out
            );
        } else {
            for t in &selected {
                println!("{}: test", t.name);
            }
            println!("\n{} tests", selected.len());
        }
        let _ = std::fs::remove_dir_all(&dir);
        return 0;
    }

    // Build the original crate. Only the compiler-generated entry unit changes:
    // it dispatches directly to checked native functions, without source wrappers.
    session.harness_tests = selected.iter().map(|test| (test.symbol, test.ignored)).collect();
    req.cmd = Cmd::Build;
    let build = driver::compile(&req, &mut session);
    if build.code != 0 {
        print!("{}", build.stdout);
        eprint!("{}", build.stderr);
        if opts.json {
            println!("{}", summary_json(&path, &hash, "failed", &[], filtered_out, start));
        }
        let _ = std::fs::remove_dir_all(&dir);
        return 101;
    }

    // Run each selected test in its own child process.
    if !opts.json {
        println!("running {} tests", selected.len());
    }
    let mut outcomes: Vec<Outcome> = Vec::new();
    for (i, t) in selected.iter().enumerate() {
        // An ignored test is reported, never run, as in rustc.
        let mut o = if t.ignored {
            Outcome {
                name: t.name.clone(),
                status: "ignored",
                duration_ms: 0.0,
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
            }
        } else {
            run_one(&bin, i, &t.name, opts.timeout)
        };
        // A `should_panic` test passes only when it panics, and its
        // output must hold the expected text when one is written.
        // A timeout stays a timeout.
        if let Some(expected) = &t.should_panic
            && o.status != "timeout"
            && o.status != "ignored"
        {
            let text_hit = expected.as_ref().is_none_or(|want| o.stdout.contains(want.as_str()) || o.stderr.contains(want.as_str()));
            o.status = if o.exit_code != Some(0) && text_hit { "pass" } else { "fail" };
        }
        if opts.json {
            println!(
                "{{\"schema\":{},\"kind\":\"test\",\"name\":{},\"status\":\"{}\",\"duration_ms\":{:.3},\"exit_code\":{},\"stdout\":{},\"stderr\":{}}}",
                diag::SCHEMA,
                diag::json_str(&o.name),
                o.status,
                o.duration_ms,
                o.exit_code.map_or("null".into(), |c| c.to_string()),
                diag::json_str(&o.stdout),
                diag::json_str(&o.stderr)
            );
        } else {
            let word = match o.status {
                "pass" => "ok",
                "timeout" => "TIMEOUT",
                "ignored" => "ignored",
                _ => "FAILED",
            };
            println!("test {} ... {} ({:.1}ms)", o.name, word, o.duration_ms);
        }
        outcomes.push(o);
    }
    let failed = outcomes.iter().filter(|o| matches!(o.status, "fail" | "timeout")).count();
    if opts.json {
        println!("{}", summary_json(&path, &hash, "ok", &outcomes, filtered_out, start));
    } else {
        if failed > 0 {
            println!("\nfailures:");
            for o in outcomes.iter().filter(|o| matches!(o.status, "fail" | "timeout")) {
                println!("---- {} ({}) ----", o.name, o.status);
                if !o.stdout.is_empty() {
                    println!("stdout:\n{}", o.stdout);
                }
                if !o.stderr.is_empty() {
                    println!("stderr:\n{}", o.stderr);
                }
            }
        }
        let passed = outcomes.iter().filter(|o| o.status == "pass").count();
        let timed_out = outcomes.iter().filter(|o| o.status == "timeout").count();
        let ignored = outcomes.iter().filter(|o| o.status == "ignored").count();
        println!(
            "\ntest result: {}. {} passed; {} failed; {} timed out; {} ignored; {} filtered out",
            if failed == 0 { "ok" } else { "FAILED" },
            passed,
            failed - timed_out,
            timed_out,
            ignored,
            filtered_out
        );
    }
    let _ = std::fs::remove_dir_all(session.obj_dir());
    let _ = std::fs::remove_dir_all(&dir);
    if failed == 0 { 0 } else { 101 }
}
