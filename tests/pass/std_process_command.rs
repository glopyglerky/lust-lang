// std::process::Command (W-SHIM phase 2): status, output, spawn with
// pipes, wait/try_wait/kill, env and current_dir, spawn errors, and
// the Debug/Display forms of Command, ExitStatus, and Output.
use std::io::{Read, Write};
use std::process::{Command, Stdio};

fn show(label: &str, r: std::io::Result<std::process::ExitStatus>) {
    match r {
        Ok(s) => println!("{label} Ok({s})"),
        Err(e) => println!("{label} Err({e} {:?} {:?})", e.kind(), e.raw_os_error()),
    }
}

fn main() {
    // Debug (shell form) and exit statuses.
    let mut c = Command::new("ls");
    c.arg("-l").arg("a b");
    println!("{c:?}");
    show("true", Command::new("true").status());
    show("false", Command::new("false").status());
    let st = Command::new("sh").arg("-c").arg("exit 42").status().unwrap();
    println!("{st} {st:?} code={:?} success={}", st.code(), st.success());
    let st = Command::new("sh").arg("-c").arg("kill -TERM $$").status().unwrap();
    println!("{st} {st:?} code={:?} success={}", st.code(), st.success());
    // output(): stdout and stderr captured, stdin empty.
    let out = Command::new("sh").arg("-c").arg("printf out; printf err >&2; read x; printf \"<%s>\" \"$x\"; exit 3").output().unwrap();
    println!("{out:?}");
    // args through an array and a Vec.
    let arr: Vec<&str> = vec!["%s-%s", "a", "b"];
    let out = Command::new("printf").args(arr).output().unwrap();
    println!("{:?}", out.stdout);
    let v = vec![String::from("%s"), String::from("vec")];
    let out = Command::new("printf").args(v).output().unwrap();
    println!("{:?}", out.stdout);
    // env: set, replace, clear; child sees the sorted merge.
    let out = Command::new("/usr/bin/env").env_clear().env("ZZZ", "1").env("AAA", "2").env("AAA", "9").output().unwrap();
    println!("{:?}", out.stdout);
    let out = Command::new("sh").arg("-c").arg("printf %s \"$MYVAR\"").env("MYVAR", "hello").output().unwrap();
    println!("{:?}", out.stdout);
    // current_dir.
    let out = Command::new("sh").arg("-c").arg("pwd").current_dir("/").output().unwrap();
    println!("{:?}", out.stdout);
    // spawn with pipes: write stdin, read stdout, wait (twice).
    let mut child = Command::new("cat").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
    println!("child id>0: {}", child.id() > 0);
    println!("{child:?}");
    child.stdin.take().unwrap().write_all(b"pipe data\n").unwrap();
    let mut s = String::new();
    child.stdout.take().unwrap().read_to_string(&mut s).unwrap();
    println!("cat: {s:?}");
    println!("wait: {}", child.wait().unwrap());
    println!("wait again: {}", child.wait().unwrap());
    // try_wait on a running child, then kill.
    let mut child = Command::new("sleep").arg("30").spawn().unwrap();
    match child.try_wait() {
        Ok(None) => println!("still running"),
        Ok(Some(s)) => println!("done {s}"),
        Err(e) => println!("err {e}"),
    }
    match child.kill() {
        Ok(()) => println!("kill ok"),
        Err(e) => println!("kill err {e}"),
    }
    println!("killed: {}", child.wait().unwrap());
    // wait_with_output.
    let out = Command::new("printf").arg("wwo").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().unwrap();
    println!("{out:?}");
    // Spawn errors.
    show("missing", Command::new("definitely-not-a-prog-xyz").status());
    show("missing-abs", Command::new("/no/such/prog").status());
    show("dir-as-prog", Command::new("/tmp").status());
    show("cwd-missing", Command::new("true").current_dir("/no/such/dir").status());
    show("nul-arg", Command::new("true").arg("a\0b").status());
    show("nul-prog", Command::new("a\0b").status());
    show("env-clear-search", Command::new("true").env_clear().status());
    // Alternate Debug.
    let mut c = Command::new("p");
    c.env_remove("HOME").env("B", "2").current_dir("/tmp").stdout(Stdio::piped());
    println!("{c:?}");
    println!("{c:#?}");
}
